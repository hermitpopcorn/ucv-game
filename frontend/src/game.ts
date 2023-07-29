import { PUBLIC_GAMESERVER_URL } from '$env/static/public';
import { toast } from '@zerodevx/svelte-toast';
import { get } from 'svelte/store';
import { websocketConnection } from '$base/stores';
import { setPlayer } from './player';
import { setOrganizer } from './organizer';

import type { Organizer, Player, WebSocketMessage } from '$base/types';

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
	console.debug(message);

	if (message.action == 'ok') {
		// None
	} else if (message.action == 'ng') {
		toast.push(message.payload, { classes: ['toast failure'] });
	} else if (message.action == 'show-message') {
		toast.push(message.payload, { classes: ['toast'] });
	} else if (message.action == 'set-player') {
		const player: Player = message.payload;
		setPlayer(player);
	} else if (message.action == 'set-organizer') {
		const organizer: Organizer = message.payload;
		setOrganizer(organizer);
	}

	if (message.responseId) {
		resolveResponseStack(message.responseId);
	}
}
