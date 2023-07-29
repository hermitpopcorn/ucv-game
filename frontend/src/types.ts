export type WebSocketConnection = {
	state: 'disconnected' | 'connected' | 'connecting' | 'error';
	connection: WebSocket | null;
};

export type WebSocketMessage = {
	responseId: string | null;
	action: string;
	payload: any;
};

export type Player = {
	id: number;
	name: string;
	score: number;
	canVote: boolean;
};

export type ActivePlayers = Array<Player>;

export type GamePhase =
	| 'standby'
	| 'show-question'
	| 'show-choices'
	| 'voting-time'
	| 'voting-locked'
	| 'show-votes'
	| 'defense'
	| 'show-results';

export type Loading = 'loading';
