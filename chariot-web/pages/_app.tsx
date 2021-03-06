import { useState } from 'react';
import '../styles/globals.css'
import styles from '../styles/Defaults.module.scss';
import type { AppProps } from 'next/app'
import { VotingGameState, GlobalContext } from '../src/contexts/GlobalContext';
import { Prompt, QuestionResult, Standing } from '../src/utils/networking';
import Logo from '../src/assets/Logo.png'
import BG from '../src/assets/BG.png'
import Image from 'next/image'
import Typewriter from 'typewriter-effect';

import NoSsr from '../src/utils/NoSsr';

function MyApp({ Component, pageProps }: AppProps) {
	const [statusMessage, setStatusMessage] = useState("");
	const [socket, setSocket] = useState<WebSocket | null>(null);
	const [uuid, setUuid] = useState("");
	const [prompt, setPrompt] = useState<Prompt | null>(null);
	const [gameState, setGameState] = useState<VotingGameState>('waiting')
	const [winner, setWinner] = useState<number | null>(null);
	const [standings, setStandings] = useState<Standing[]>([]);
	const [totalConnected, setTotalConnected] = useState(1);
	const [countdownTime, setCountdownTime] = useState<Date | null>(null);
	const [optionResults, setOptionResults] = useState<QuestionResult[]>([]);

	const funnyPhrases = ["I prefer folding",
		"Hold onto your seats",
		"Why stand when you can sit",
		"Nascar ain't got squat on this",
		"UCSD Surplus sells great chairs",
		"Cushions are overrated",
		"Mesh chairs are the best"
	];

	const displayStatusMessage = statusMessage.length > 0;
	const ratio = Logo.width / Logo.height;

	return (
		<GlobalContext.Provider value={{
			statusMessage,
			setStatusMessage,
			socket,
			setSocket,
			uuid,
			setUuid,
			prompt,
			setPrompt,
			gameState,
			setGameState,
			winner,
			setWinner,
			standings,
			setStandings,
			totalConnected,
			setTotalConnected,
			countdownTime,
			setCountdownTime,
			optionResults,
			setOptionResults
		}}>
			<div className={styles.backgroundImage} style={{ backgroundImage: `url(${BG.src})` }}>
				<div className={styles.main}>
					<div className={styles.header}>
						<div className={styles.headerImage}>
							<Image alt="Chairot" src={Logo} />
						</div>
						<NoSsr>
							<div className={styles.headerContainer}>
								<div className={styles.headerText}>
									<Typewriter options={{
										strings: displayStatusMessage ? statusMessage : funnyPhrases,
										autoStart: true,
										loop: !displayStatusMessage,
										delay: 60,
									}
									} />
								</div>
							</div>
						</NoSsr>
					</div>
					<div className={styles.rest}>
						<Component {...pageProps} />
					</div>
				</div>
			</div>
		</GlobalContext.Provider>
	)
}

export default MyApp
