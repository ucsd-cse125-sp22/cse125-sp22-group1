# chariot-server

Responsible for chariot game simulation and networking with clients.

This server will need to expose two API's:
1. a custom TCP connection to locally networked clients
2. an internet-facing websocket connection to audience web clients

### running the server
```bash
cd chariot-server
# make source changes...
cargo run
```

