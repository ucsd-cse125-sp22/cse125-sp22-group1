import type { NextPage } from 'next'
import Router, { useRouter } from 'next/router'
import { useContext, useEffect, useState } from 'react'
import { Button } from '../src/components/Button'
import { GlobalContext } from '../src/contexts/GlobalContext'
import { handleSocket } from '../src/utils/networking'
import styles from '../styles/Index.module.scss';

const Home: NextPage = () => {
	const router = useRouter();
	const queryIp = router.query.ip;
	const context = useContext(GlobalContext);

	useEffect(() => {
		if (!queryIp) {
			router.push(`/?ip=127.0.0.1:2334`); // you need an active socket to be here
		}
	})

	const connectToWebSocket = () => {
		const sock = new WebSocket(`ws://${queryIp}`);
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
			<div className={styles.textBox}>
				Joining game @ {queryIp}
			</div>

			<Button text='Join active game' onClick={() => {
				connectToWebSocket();
			}} />
		</>
	)
}

export default Home
