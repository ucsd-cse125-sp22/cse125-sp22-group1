import { useState } from 'react';
import '../styles/globals.css'
import styles from '../styles/Defaults.module.scss';
import type { AppProps } from 'next/app'
import { GlobalContext } from '../src/contexts/GlobalContext';

function MyApp({ Component, pageProps }: AppProps) {
	const [statusMessage, setStatusMessage] = useState("i prefer folding");
	const [socket, setSocket] = useState<WebSocket | null>(null);
	const [uuid, setUuid] = useState("");

	return (
		<GlobalContext.Provider value={{
			statusMessage,
			setStatusMessage,
			socket,
			setSocket,
			uuid,
			setUuid
		}}>
			<div className={styles.main}>
				<div className={styles.header}>
					<h1 className={styles.headerText}>Chairiot</h1>
					<p>{statusMessage}</p>
				</div>
				<div className={styles.rest}>
					<Component {...pageProps} />
				</div>
			</div>
		</GlobalContext.Provider>
	)
}

export default MyApp
