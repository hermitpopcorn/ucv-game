import { writable, type Writable } from 'svelte/store';
import type { GameState, Organizer, Player, WebSocketConnection } from '$base/types';

export const browserEnv: Writable<{ server: string | null }> = writable({
	server: null,
});

export const websocketConnection: Writable<WebSocketConnection> = writable({
	state: 'disconnected',
	connection: null,
});

export const player: Writable<Player | null> = writable(null);

export const organizer: Writable<Organizer | null> = writable(null);

export const gameState: Writable<GameState | null> = writable(null);
