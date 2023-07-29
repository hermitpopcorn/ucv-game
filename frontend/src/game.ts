import { PUBLIC_GAMESERVER_URL } from '$env/static/public';
import { toast } from '@zerodevx/svelte-toast';
import { get } from 'svelte/store';
import { v4 as generateUuid } from 'uuid';
import { websocketConnection } from '$base/stores';
import { gameState as gameStateStore } from '$base/stores';
import { setPlayer } from '$base/player';
import { setOrganizer } from '$base/organizer';

import type { GameState, WebSocketMessage } from '$base/types';

const awaitResponseStack: Map<string, () => void> = new Map();

export function pushResponseStack(id: string, callback: () => void) {
	awaitResponseStack.set(id, callback);
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
		const socket = new WebSocket(PUBLIC_GAMESERVER_URL);

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
	} else if (message.action == 'set-game-state') {
		setGameState(message.payload);
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

export function setGameState(gameState: GameState) {
	gameStateStore.set(gameState);
}
