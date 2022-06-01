import type { GetServerSideProps, NextPage } from 'next'
import Router, { useRouter } from 'next/router'
import { useContext, useEffect, useState } from 'react'
import publicIp from 'public-ip';
import { Button } from '../src/components/Button'
import { GlobalContext } from '../src/contexts/GlobalContext'
import { handleSocket } from '../src/utils/networking'
import styles from '../styles/Index.module.scss';
import { internalIpV4 } from 'internal-ip';

const Home: NextPage = () => {
	const router = useRouter();
	const queryIp = router.query.ip;
	const context = useContext(GlobalContext);

	const connectToWebSocket = () => {
		const sock = new WebSocket(`ws://${queryIp}`);
		sock.onopen = () => {
			context.setSocket(sock);
			(window as any).socket = sock;
			Router.push(`/game?ip=${queryIp}`);
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
		<div className={styles.container}>
			<Button text='Join Game' onClick={() => {
				connectToWebSocket();
			}} />
		</div>
	)
}

export const getServerSideProps: GetServerSideProps = async ({ query, req }) => {
	if (!query.ip) {
		const port = req.headers.host?.split(":")[1] || 80
		const ip = await internalIpV4()
		return {
			redirect: {
				permanent: false,
				destination: `http://${ip}:${port}/?ip=${ip}:2334`
			}
		};
	}
	return { props: {} };
}

export default Home


