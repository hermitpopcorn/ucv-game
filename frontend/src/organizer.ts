import { v4 as generateUuid } from 'uuid';
import { getWebsocketConnection, pushResponseStack } from '$base/game';
import { organizer as organizerStore } from '$base/stores';

import type { Organizer } from './types';

export function setOrganizer(organizer: Organizer) {
	organizerStore.set(organizer);
}

export function login(password: string): Promise<void> {
	return new Promise((resolve) => {
		const socket = getWebsocketConnection();
		const responseId = generateUuid();
		socket.send(
			JSON.stringify({
				responseId,
				action: 'organizerLogin',
				payload: password,
			}),
		);

		pushResponseStack(responseId, resolve);
	});
}
