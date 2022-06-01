import React from 'react';
import { Prompt, Standing } from '../utils/networking';

export type VotingGameState = 'voting' | 'winner' | 'waiting';

export type GlobalContextType = {
	gameState: VotingGameState,
	setGameState: React.Dispatch<React.SetStateAction<VotingGameState>>,

	prompt: Prompt | null,
	setPrompt: React.Dispatch<React.SetStateAction<Prompt | null>>,

	statusMessage: string;
	setStatusMessage: React.Dispatch<React.SetStateAction<string>>;

	socket: WebSocket | null;
	setSocket: React.Dispatch<React.SetStateAction<WebSocket | null>>;

	uuid: string;
	setUuid: React.Dispatch<React.SetStateAction<string>>;

	winner: number | null;
	setWinner: React.Dispatch<React.SetStateAction<number | null>>;

	standings: Standing[],
	setStandings: React.Dispatch<React.SetStateAction<Standing[]>>;

	totalConnected: number;
	setTotalConnected: React.Dispatch<React.SetStateAction<number>>;
};

export const GlobalContext = React.createContext<GlobalContextType>(null as any);