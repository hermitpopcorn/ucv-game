import { v4 as generateUuid } from 'uuid';
import { getWebsocketConnection, pushResponseStack } from './game';
import { player as playerStore } from '$base/stores';

import type { Player } from '$base/types';

export function setPlayer(player: Player) {
	playerStore.set(player);
}

export function login(name: string): Promise<void> {
	return new Promise((resolve) => {
		const socket = getWebsocketConnection();
		const responseId = generateUuid();
		socket.send(
			JSON.stringify({
				responseId,
				action: 'login-player',
				payload: name,
			}),
		);

		pushResponseStack(responseId, resolve);
	});
}
