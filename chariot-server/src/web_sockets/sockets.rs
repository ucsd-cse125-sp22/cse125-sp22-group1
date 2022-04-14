use std::fmt::format;

use futures::{FutureExt, StreamExt};
use serde::Deserialize;
use serde_json::from_str;
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};

use super::{Client, Clients};

#[derive(Deserialize, Debug)]
pub struct TopicsRequest {
    topics: Vec<String>,
}

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split(); // split into a stream (sender) and a sink (receiver)
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let client_id = client.user_id.clone();

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        // any message sent to the sink goes to the stream
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender);
    clients.write().await.insert(id.clone(), client);

    println!("{} connected", id);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &clients, client_id.clone()).await;
    }
    // if we get here, the client is gone / isn't sending messages, we gotta remove it from our set of clients

    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients, client_id: String) {
    println!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    println!("total # of clients {}", clients.read().await.len());

    if message == "ping" || message == "ping\n" {
        clients
            .read()
            .await
            .iter()
            .filter(|(_, client)| client.user_id == client_id)
            .for_each(|(_, client)| {
                if let Some(sender) = &client.sender {
                    let _ = sender.send(Ok(Message::text("pong")));
                }
            });
        return;
    }

    let topics_req: TopicsRequest = match from_str(&message) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error while parsing message to topics request: {}", e);
            return;
        }
    };
    println!("{:?}", topics_req);

    let mut locked = clients.write().await;
    if let Some(v) = locked.get_mut(id) {
        println!("did we even get here? {}", id);
        v.topics = topics_req.topics;
    }

    println!("why can we not send anything back");
    println!("total # of clients {}", clients.read().await.len());
    clients
        .read()
        .await
        .iter()
        .filter(|(_, client)| client.user_id == client_id)
        .for_each(|(_, client)| {
            println!("we found a client");
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text("nice, we just subbed to a topic!")));
                let _ = sender.send(Ok(Message::text(format!("{:?}", client.topics))));
            }
        });
}
