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
export interface WSAudienceBoundMessage {
	Prompt?: { question: Prompt, time_until_vote_end: number }, // Question, 4 Answer Choices
	Winner?: number// The winning choice (tuple index)
	Assignment?: string, // Sends a uuid that the server will use to identify the client
	Standings?: [Standing], // state of the game
	AudienceCount: number, // the number of audience members connected
}

export interface WSServerBoundMessage {
	Vote?: [string, number];
}

export const handleSocket = (context: GlobalContextType, msg: MessageEvent) => {
	const message: WSAudienceBoundMessage = JSON.parse(msg.data);

	if (message.Assignment !== undefined) {
		context.setUuid(message.Assignment);
	} else if (message.Winner !== undefined) {
		context.setWinner(message.Winner);
	} else if (message.Prompt !== undefined) {
		context.setPrompt(message.Prompt.question);
		context.setStatusMessage(message.Prompt.question.prompt);
		console.log(message.Prompt.time_until_vote_end);
		context.setWinner(null);
	} else if (message.Standings !== undefined) {
		context.setStandings(message.Standings);
	} else if (message.AudienceCount !== undefined) {
		context.setTotalConnected(message.AudienceCount);
	} else {
		console.log("new data type");
		console.log(message);
	}
}

export const sendMessage = (context: GlobalContextType, message: WSServerBoundMessage) => {
	context.socket?.send(JSON.stringify(message));
}