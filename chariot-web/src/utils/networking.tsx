import { GlobalContextType } from "../contexts/GlobalContext"

export type Prompt = [string, [string, string, string, string]];
export interface WSAudienceBoundMessage {
	Prompt?: Prompt, // Question, 4 Answer Choices
	Winner?: number// The winning choice (tuple index)
	Assignment?: string, // Sends a uuid that the server will use to identify the client
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
		context.setPrompt(message.Prompt);
		context.setStatusMessage(message.Prompt[0]);
	} else {
		console.log("new data type");
		console.log(message);
	}
}

export const sendMessage = (context: GlobalContextType, message: WSServerBoundMessage) => {
	context.socket?.send(JSON.stringify(message));
}