import { GlobalContextType } from "../contexts/GlobalContext"

export interface Prompt {
	prompt: string;
	options: { label: string, action: string }[]
}

export interface Standing {
	name: string,
	chair: string,
	rank: number,
	lap: number,
}

export interface QuestionResult {
	label: string,
	percentage: number,
}

export interface WSAudienceBoundMessage {
	Prompt?: {
		question: Prompt,
		vote_close_time: number
	}, // Question, 4 Answer Choices
	Winner?: {
		choice: number,
		vote_effect_time: number,
		option_results: QuestionResult[]
	}// The winning choice (tuple index)
	Assignment?: string, // Sends a uuid that the server will use to identify the client
	Standings?: [Standing], // state of the game
	AudienceCount?: number, // the number of audience members connected
	Countdown?: {
		time: number
	} // the time left for something (state independent, used for new connections)
}

export interface WSServerBoundMessage {
	Vote?: [string, number];
}

export const handleSocket = (context: GlobalContextType, msg: MessageEvent) => {
	const message: WSAudienceBoundMessage = JSON.parse(msg.data);

	if (message.Assignment !== undefined) {
		context.setUuid(message.Assignment);
	} else if (message.Winner !== undefined) {
		context.setWinner(message.Winner.choice);
		context.setCountdownTime(new Date(message.Winner.vote_effect_time));
		context.setOptionResults(message.Winner.option_results);
		context.setGameState("winner");
	} else if (message.Prompt !== undefined) {
		context.setPrompt(message.Prompt.question);
		context.setStatusMessage(message.Prompt.question.prompt);
		context.setCountdownTime(new Date(message.Prompt.vote_close_time));
		context.setGameState("voting");
		context.setWinner(null);
	} else if (message.Standings !== undefined) {
		context.setStandings(message.Standings);
	} else if (message.AudienceCount !== undefined) {
		context.setTotalConnected(message.AudienceCount);
	} else if (message.Countdown !== undefined) {
		context.setCountdownTime(new Date(message.Countdown.time));
		context.setGameState('waiting');
	} else {
		console.log("new data type");
		console.log(message);
	}
}

export const sendMessage = (context: GlobalContextType, message: WSServerBoundMessage) => {
	context.socket?.send(JSON.stringify(message));
}