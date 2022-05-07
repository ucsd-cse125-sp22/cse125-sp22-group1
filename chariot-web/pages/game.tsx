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
	const { socket } = context;

	useEffect(() => {
		if (socket == null) {
			router.push("/"); // you need an active socket to be here
		}
	})

	if (socket == null) {
		return <></>;
	}

	socket.onmessage = (msg) => {
		handleSocket(context, msg);
	}

	return (<>
		<Button text={showStandings ? "show standings" : "hide standings"} onClick={() => { setShowStandings(!showStandings) }} style='minimal' />
		<br />
		{!showStandings &&
			<Grid>
				<Button text="option 1" onClick={() => {
					sendMessage(context, { Vote: [context.uuid, 0] })
					socket.send("option 1");
				}} />
				<Button text="option 2" onClick={() => {
					sendMessage(context, { Vote: [context.uuid, 1] })
					socket.send("option 2");
				}} />
				<Button text="option 3" onClick={() => {
					sendMessage(context, { Vote: [context.uuid, 2] })
					socket.send("option 3");
				}} />
				<Button text="option 4" onClick={() => {
					sendMessage(context, { Vote: [context.uuid, 3] })
					socket.send("option 4");
				}} />
			</Grid>
		}
		{showStandings &&
			<Standings />}
	</>)
}

export default Game;