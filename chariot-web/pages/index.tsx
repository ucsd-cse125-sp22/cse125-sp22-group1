import type { NextPage } from 'next'
import Head from 'next/head'
import Image from 'next/image'
import { MutableRefObject, useRef, useState } from 'react'
import styles from '../styles/Home.module.css'

const Home: NextPage = () => {
	const socket: MutableRefObject<WebSocket | null> = useRef(null);
	const [connected, setConnected] = useState(false);

	const connectToWebSocket = () => {
		socket.current = new WebSocket('ws://127.0.0.1:9001');
		socket.current.onopen = () => {
			socket.current?.send("Hello server!");
			setConnected(true);
		}

		socket.current.onmessage = (msg) => {
			console.log("we got a message!");
			alert(msg.data);
		}
	}

	const leaveWebSocket = () => {
		socket.current?.close();
		socket.current = null;
		setConnected(false);
	}

	const sendMessage = () => {
		const message = prompt("what message do you want to send?");
		if (message !== null) {
			socket.current?.send(message);
		}
	}

	return (
		<div className={styles.container}>
			<Head>
				<title>Chairiot</title>
				<link rel="icon" href="/favicon.ico" />
			</Head>

			<main className={styles.main}>
				<h1 className={styles.title}>
					Welcome to <a href="https://chairiots.notion.site/Chairioteers-bf2521783c0a46f7b06b0f072779be0f">Chairiot!</a>
				</h1>

				{connected &&
					<p className={styles.description}>
						<button onClick={sendMessage}>Send Ping</button>
					</p>
				}

				{!connected &&
					<div className={styles.grid}>
						<a href="#" onClick={connectToWebSocket} className={styles.card}>
							<h2>Join Active Game &rarr;</h2>
							<p>Attempt to connect to the web socket.</p>
						</a>
					</div>
				}
				{connected &&
					<div className={styles.grid}>
						<a href="#" onClick={leaveWebSocket} className={styles.card}>
							<h2>Leave Current Game &rarr;</h2>
							<p>Disconnect from the web socket.</p>
						</a>
					</div>
				}
			</main>

			<footer className={styles.footer}>
				<a
					href="https://vercel.com?utm_source=create-next-app&utm_medium=default-template&utm_campaign=create-next-app"
					target="_blank"
					rel="noopener noreferrer"
				>
					Powered by{' '}
					<span className={styles.logo}>
						<Image src="/vercel.svg" alt="Vercel Logo" width={72} height={16} />
					</span>
				</a>
			</footer>
		</div>
	)
}

export default Home
