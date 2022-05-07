import React from 'react';

type GlobalContextType = {
	statusMessage: string;
	setStatusMessage: React.Dispatch<React.SetStateAction<string>>;
	socket: WebSocket | null;
	setSocket: React.Dispatch<React.SetStateAction<WebSocket | null>>;
};

export const GlobalContext = React.createContext<GlobalContextType>(null as any);