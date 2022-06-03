import { NextPage } from "next"
import Router, { useRouter } from "next/router";
import { useContext, useEffect, useState } from "react";
import { Button } from "../src/components/Button";
import Grid from "../src/components/Grid/Grid";
import Standings from "../src/components/Standings";
import { GlobalContext } from "../src/contexts/GlobalContext";
import { handleSocket, sendMessage } from "../src/utils/networking";
import styles from './Game.module.scss';
import AudienceIcon from '../src/assets/Audience.png'
import Image from 'next/image';
import { toPercentage } from "../src/utils/other";

const Game: NextPage = () => {
	const [showStandings, setShowStandings] = useState(true);
	const router = useRouter();
	const context = useContext(GlobalContext);
	const [selectedIdx, setSelectedIdx] = useState<number | null>(null);

	const { socket, uuid, prompt, winner, totalConnected, countdownTime, gameState, optionResults } = context;
	const othersConnected = totalConnected - 1;

	useEffect(() => {
		if (winner !== null) {
			setSelectedIdx(null);
		}
	}, [winner]);

	useEffect(() => {
		if (socket == null) {
			router.push(`/?ip=${router.query.ip}`); // you need an active socket to be here
		}
	})

	if (socket == null) {
		return <></>;
	}

	socket.onerror = (err) => {
		if (err.type === 'error') {
			alert("Failed to connect to server. Is it running?");
			router.push(`/?ip=${router.query.ip}`);
			context.setStatusMessage("");
		}
	}

	socket.onclose = () => {
		alert("you lost connection!");
		router.push(`/?ip=${router.query.ip}`);
		context.setStatusMessage("");
	}

	socket.onmessage = (msg) => {
		handleSocket(context, msg);
	}

	const timeLeft = countdownTime ? countdownTime.getSeconds() - new Date().getSeconds() : -1;
	const timeLeftText = gameState === 'voting' ? `Voting ends in ${timeLeft}s` : gameState === 'winner' ? `${timeLeft}s until effects subside` : `${timeLeft}s until next vote`

	return (<div className={styles.container}>
		<div className={styles.blockText}>
			<p>{showStandings ? "Standings" : (timeLeft >= 0) ? timeLeftText : "Waiting for Next Vote"}</p>
		</div>
		{!showStandings && prompt !== null && gameState !== 'waiting' &&
			<div className={styles.buttonLayout}>
				{prompt.options.map((({ label }, choice) => {
					const labelText = `${label}${(gameState === 'winner' && optionResults?.length === prompt.options.length) ? " — " + toPercentage(optionResults[choice].percentage) : ""}`
					return (
						<Button width="100%" clickable={winner === null} state={choice === winner ? 'voted' : choice === selectedIdx ? 'selected' : 'unselected'} key={choice} text={labelText} onClick={() => {
							if (winner === null) {
								sendMessage(context, { Vote: [uuid, choice] })
								setSelectedIdx(choice);
							}
						}} />
					)
				}))}
				{prompt.options.length === 0 && <p>New Vote Coming Soon</p>}
			</div>
		}
		{showStandings &&
			<Standings />}

		<div className={styles.standingsButton}>
			<Button width="80%" text={showStandings ? "hide standings" : "see standings"} onClick={() => { setShowStandings(!showStandings) }} style='minimal' />
			<div className={styles.liveAudience}>
				<Image src={AudienceIcon} height="32.56" alt="audience icon" />
				<p>{othersConnected} Other{othersConnected !== 1 && 's'} Online</p>
			</div>
		</div>

	</div>)
}

export default Game;