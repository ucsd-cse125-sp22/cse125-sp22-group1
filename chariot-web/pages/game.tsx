import { NextPage } from "next"
import Router, { useRouter } from "next/router";
import { useContext, useEffect, useState } from "react";
import { Button } from "../src/components/Button";
import Grid from "../src/components/Grid";
import Standings from "../src/components/Standings";
import { GlobalContext } from "../src/contexts/GlobalContext";

const Game: NextPage = () => {
	const [showStandings, setShowStandings] = useState(false);
	const router = useRouter();
	const { socket } = useContext(GlobalContext);

	useEffect(() => {
		if (socket == null) {
			router.push("/"); // you need an active socket to be here
		}
	})

	if (socket == null) {
		return <></>;
	}

	socket.onmessage = (msg) => {
		console.log("game: ");
		console.log(msg.data);
	}

	return (<>
		<Button text={showStandings ? "show standings" : "hide standings"} onClick={() => { setShowStandings(!showStandings) }} style='minimal' />
		<br />
		{!showStandings &&
			<Grid>
				<Button text="option 1" onClick={() => {
					socket.send("option 1");
				}} />
				<Button text="option 2" onClick={() => {
					socket.send("option 2");
				}} />
				<Button text="option 3" onClick={() => {
					socket.send("option 3");
				}} />
				<Button text="option 4" onClick={() => {
					socket.send("option 4");
				}} />
			</Grid>
		}
		{showStandings &&
			<Standings />}
	</>)
}

export default Game;