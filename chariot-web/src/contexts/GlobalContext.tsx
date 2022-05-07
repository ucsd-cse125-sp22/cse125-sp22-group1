import React from 'react';

export type GlobalContextType = {
	statusMessage: string;
	setStatusMessage: React.Dispatch<React.SetStateAction<string>>;
	socket: WebSocket | null;
	setSocket: React.Dispatch<React.SetStateAction<WebSocket | null>>;
	uuid: string;
	setUuid: React.Dispatch<React.SetStateAction<string>>;
};

export const GlobalContext = React.createContext<GlobalContextType>(null as any);