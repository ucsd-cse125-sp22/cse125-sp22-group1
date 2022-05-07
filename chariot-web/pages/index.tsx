import type { NextPage } from 'next'
import Router from 'next/router'
import { useContext, useState } from 'react'
import { Button } from '../src/components/Button'
import { GlobalContext } from '../src/contexts/GlobalContext'

const Home: NextPage = () => {
	const context = useContext(GlobalContext);

	const connectToWebSocket = () => {
		const sock = new WebSocket('ws://127.0.0.1:9001');
		sock.onopen = () => {
			sock.send("Hello server!");
			context.setSocket(sock);
			(window as any).socket = sock;
			Router.push("/game");
		}

		sock.onerror = (err) => {
			if (err.type === 'error') {
				alert("Failed to connect to server. Is it running?");
			}
		}

		sock.onmessage = (msg) => {
			console.log(msg);
		}

	}

	return (
		<>
			<Button text='Join active game' state='unselected' onClick={() => {
				connectToWebSocket();
			}} />
		</>
	)
}

export default Home
