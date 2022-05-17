import type { NextPage } from 'next'
import Router from 'next/router'
import { useContext, useState } from 'react'
import { Button } from '../src/components/Button'
import { GlobalContext } from '../src/contexts/GlobalContext'
import { handleSocket } from '../src/utils/networking'
import styles from '../styles/Index.module.scss';

const Home: NextPage = () => {
	const context = useContext(GlobalContext);
	const [ip, setIp] = useState("128.54.70.27:2334");

	const connectToWebSocket = () => {
		const sock = new WebSocket(`ws://${ip}`);
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
				<input value={ip} onChange={e => setIp(e.target.value)} />
			</div>

			<Button text='Join active game' onClick={() => {
				connectToWebSocket();
			}} />
		</>
	)
}

export default Home
