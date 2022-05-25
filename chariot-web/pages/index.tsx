import type { GetServerSideProps, NextPage } from 'next'
import Router, { useRouter } from 'next/router'
import { useContext, useEffect, useState } from 'react'
import publicIp from 'public-ip';
import { Button } from '../src/components/Button'
import { GlobalContext } from '../src/contexts/GlobalContext'
import { handleSocket } from '../src/utils/networking'
import styles from '../styles/Index.module.scss';

const Home: NextPage = () => {
	const router = useRouter();
	const queryIp = router.query.ip;
	const context = useContext(GlobalContext);

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

export const getServerSideProps: GetServerSideProps = async ({ query }) => {
	if (!query.ip) {
		const ip = await publicIp.v4()
		return {
			redirect: {
				permanent: false,
				destination: `?ip=${ip}:2334`
			}
		};
	}
	return { props: {} };
}

export default Home


