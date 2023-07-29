import { writable, type Writable } from 'svelte/store';
import type { GameState, Organizer, Player, WebSocketConnection } from '$base/types';

export const websocketConnection: Writable<WebSocketConnection> = writable({
	state: 'disconnected',
	connection: null,
});

export const player: Writable<Player | null> = writable(null);

export const organizer: Writable<Organizer | null> = writable(null);

export const gameState: Writable<GameState | null> = writable(null);
