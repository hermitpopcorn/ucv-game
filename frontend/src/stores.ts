import { writable, type Writable } from 'svelte/store';
import type { Player, WebSocketConnection } from './types';

export const websocketConnection: Writable<WebSocketConnection> = writable({
	state: 'disconnected',
	connection: null,
});

export const player: Writable<Player | null> = writable(null);
