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

export type Organizer = {
	name: string;
};

export type RoundState =
	| 'standby'
	| 'show-question'
	| 'show-choices'
	| 'voting-time'
	| 'voting-locked'
	| 'show-votes'
	| 'defense'
	| 'show-results';

export type Round = {
	id: number;
	number: number;
	phase: number;
	state: RoundState;
	question: string;
	choiceA: string;
	choiceB: string;
};

export type ChoiceOption = 'a' | 'b';

export type Choice = {
	id: number;
	option: ChoiceOption;
	lie: boolean;
};

export type GameState = {
	round: Round;
	players: ActivePlayers;
	choices: Map<number, Choice>;
};
