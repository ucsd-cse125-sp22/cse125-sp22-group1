import { NextPage } from "next"
import Router, { useRouter } from "next/router";
import { useContext, useEffect, useState } from "react";
import { Button } from "../src/components/Button";
import Grid from "../src/components/Grid/Grid";
import Standings from "../src/components/Standings";
import { GlobalContext } from "../src/contexts/GlobalContext";
import { handleSocket, sendMessage } from "../src/utils/networking";

const Game: NextPage = () => {
	const [showStandings, setShowStandings] = useState(false);
	const router = useRouter();
	const context = useContext(GlobalContext);
	const [selectedIdx, setSelectedIdx] = useState<number | null>(null);

	const { socket, uuid, prompt, winner } = context;

	useEffect(() => {
		if (socket == null) {
			router.push("/"); // you need an active socket to be here
		}
	})

	if (socket == null) {
		return <></>;
	}

	socket.onerror = (err) => {
		console.log('re')
		if (err.type === 'error') {
			alert("Failed to connect to server. Is it running?");
		}
	}
	socket.onmessage = (msg) => {
		handleSocket(context, msg);
	}

	return (<>
		<Button text={showStandings ? "hide standings" : "show standings"} onClick={() => { setShowStandings(!showStandings) }} style='minimal' />
		<br />
		{!showStandings && prompt !== null &&
			<Grid>
				{prompt.options.map((({ label }, choice) => (
					<Button state={choice === winner ? 'voted' : choice === selectedIdx ? 'selected' : 'unselected'} key={choice} text={label} onClick={() => {
						if (winner === null) {
							sendMessage(context, { Vote: [uuid, choice] })
							setSelectedIdx(choice);
						}
					}} />
				)))}
			</Grid>
		}
		{showStandings &&
			<Standings />}
	</>)
}

export default Game;