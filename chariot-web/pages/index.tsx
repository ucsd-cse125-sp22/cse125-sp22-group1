import type { NextPage } from 'next'
import Router from 'next/router'
import { useContext, useState } from 'react'
import { Button } from '../src/components/Button'
import { GlobalContext } from '../src/contexts/GlobalContext'
import { handleSocket, WS_SERVER } from '../src/utils/networking'

const Home: NextPage = () => {
	const context = useContext(GlobalContext);

	const connectToWebSocket = () => {
		const sock = new WebSocket(WS_SERVER);
		sock.onopen = () => {
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
			handleSocket(context, msg);
		}
	}

	return (
		<>
			<Button text='Join active game' onClick={() => {
				connectToWebSocket();
			}} />
		</>
	)
}

export default Home
