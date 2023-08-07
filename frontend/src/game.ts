import { PUBLIC_GAMESERVER_URL } from '$env/static/public';
import { toast } from '@zerodevx/svelte-toast';
import { get } from 'svelte/store';
import { v4 as generateUuid } from 'uuid';
import { browserEnv, websocketConnection } from '$base/stores';
import { gameState as gameStateStore } from '$base/stores';
import { setPlayer, setPlayerIfSelf } from '$base/player';
import { setOrganizer } from '$base/organizer';

import type { Choice, ChoiceMap, GameState, Player, Round, WebSocketMessage } from '$base/types';

const awaitResponseStack: Map<string, () => void> = new Map();

export function pushResponseStack(id: string, callback: () => void, timeoutCallback?: () => void) {
	awaitResponseStack.set(id, callback);

	if (timeoutCallback) {
		setTimeout(() => {
			if (!awaitResponseStack.get(id)) {
				return;
			}

			timeoutCallback();
		}, 5000);
	}
}

function resolveResponseStack(id: string) {
	const resolver = awaitResponseStack.get(id);
	if (resolver === undefined) {
		return;
	}

	resolver();
}

export async function connect() {
	websocketConnection.update((wsc) => {
		wsc.connection = null;
		wsc.state = 'connecting';
		return wsc;
	});
	const connectingToast = toast.push('Connecting to game server...', { initial: 0 });

	try {
		const websocket = await connectWebsocket();
		websocketConnection.update((wsc) => {
			wsc.connection = websocket;
			wsc.state = 'connected';
			return wsc;
		});
		toast.pop(connectingToast);
		toast.push('Successfully connected to game server.', {
			classes: ['toast success'],
		});
	} catch {
		websocketConnection.update((wsc) => {
			wsc.state = 'error';
			return wsc;
		});
		toast.pop(connectingToast);
		toast.push('Failed connecting to game server.', {
			classes: ['toast failure'],
		});
	}
}

function connectWebsocket(): Promise<WebSocket> {
	return new Promise((resolve, reject) => {
		const defaultGameserverUrl = PUBLIC_GAMESERVER_URL;
		const overrideServer = get(browserEnv).server;

		const socket = new WebSocket('ws://' + (overrideServer ?? defaultGameserverUrl));

		socket.onerror = () => {
			console.error('WebSocket connection errored.');
			reject();
		};

		socket.onopen = () => {
			console.info('WebSocket connection established.');
			resolve(socket);
		};

		socket.onmessage = (event) => {
			const parsed: WebSocketMessage = JSON.parse(event.data);
			handleMessage(parsed);
		};
	});
}

export function getWebsocketConnection(): WebSocket {
	const socket = get(websocketConnection);
	if (socket.state !== 'connected' || !socket.connection) {
		throw new Error('Not connected.');
	}

	return socket.connection;
}

function handleMessage(message: WebSocketMessage) {
	console.debug(message.action, message.payload);

	if (message.action == 'ok') {
		// None
	} else if (message.action == 'ng') {
		toast.push(message.payload, { classes: ['toast failure'] });
	} else if (message.action == 'show-message') {
		toast.push(message.payload, { classes: ['toast'] });
	} else if (message.action == 'set-player') {
		setPlayer(message.payload);
	} else if (message.action == 'set-organizer') {
		setOrganizer(message.payload);
	} else if (message.action == 'refresh-active-players-list') {
		setActivePlayers(message.payload);
	} else if (message.action == 'update-player') {
		updatePlayer(message.payload);
	} else if (message.action == 'set-round') {
		setRound(message.payload);
	} else if (message.action == 'set-game-state') {
		setGameState(message.payload);
	} else if (message.action == 'set-player-choice') {
		setPlayerChoice(message.payload);
	} else if (message.action == 'set-choices') {
		setChoicesMap(message.payload);
	}

	if (message.responseId) {
		resolveResponseStack(message.responseId);
	}
}

export function getGameState(): Promise<void> {
	return new Promise((resolve) => {
		const socket = getWebsocketConnection();
		const responseId = generateUuid();
		socket.send(
			JSON.stringify({
				responseId,
				action: 'get-game-state',
			}),
		);

		pushResponseStack(responseId, resolve);
	});
}

function getBlankGameState(): GameState {
	return {
		round: null,
		players: [],
		choices: new Map(),
	};
}

export function setActivePlayers(activePlayersList: Array<Player>) {
	gameStateStore.update((gameState) => {
		if (gameState === null) {
			gameState = getBlankGameState();
		}

		gameState.players = activePlayersList;
		return gameState;
	});
}

export function updatePlayer(player: Player) {
	gameStateStore.update((gameState) => {
		if (gameState === null) {
			gameState = getBlankGameState();
		}

		for (let i = 0; i < gameState.players.length; i++) {
			if (gameState.players[i].id == player.id) {
				gameState.players[i] = player;
			}
		}
		return gameState;
	});

	setPlayerIfSelf(player);
}

function setRound(round: Round) {
	gameStateStore.update((gameState) => {
		if (gameState === null) {
			gameState = getBlankGameState();
		}

		gameState.round = round;
		return gameState;
	});
}

function convertChoicesObjecToMap(choices: object): ChoiceMap {
	const map: Map<number, Choice> = new Map();
	if (choices) {
		for (const c of Object.entries(choices)) {
			map.set(Number(c[0]), c[1]);
		}
	}

	return map;
}

export function setGameState(gameState: GameState) {
	const choicesMap = convertChoicesObjecToMap(gameState.choices);

	gameState.choices = choicesMap;
	gameStateStore.set(gameState);
}

export function setPlayerChoice({ player, choice }: { player: Player; choice: Choice }) {
	gameStateStore.update((gameState) => {
		if (gameState === null) {
			gameState = getBlankGameState();
		}

		gameState.choices.set(player.id, choice);
		return gameState;
	});
}

export function setChoicesMap(choicesMap: object) {
	const mapped = convertChoicesObjecToMap(choicesMap);

	gameStateStore.update((gameState) => {
		if (gameState === null) {
			gameState = getBlankGameState();
		}

		gameState.choices = mapped;
		return gameState;
	});
}
