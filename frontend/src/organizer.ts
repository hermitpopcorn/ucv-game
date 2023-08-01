import { v4 as generateUuid } from 'uuid';
import { getWebsocketConnection, pushResponseStack } from '$base/game';
import { organizer as organizerStore } from '$base/stores';

import type { Organizer, Round } from './types';
import { toast } from '@zerodevx/svelte-toast';

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
				action: 'login-organizer',
				payload: password,
			}),
		);

		pushResponseStack(responseId, resolve);
	});
}

export function updateRound(round: Round): Promise<void> {
	const updatingToast = toast.push('Updating round data...', { initial: 0 });

	return new Promise((resolve) => {
		const socket = getWebsocketConnection();
		const responseId = generateUuid();
		socket.send(
			JSON.stringify({
				responseId,
				action: 'set-round',
				payload: round,
			}),
		);

		pushResponseStack(responseId, () => {
			toast.pop(updatingToast);
			toast.push('Round updated.', {
				classes: ['toast success'],
			});

			resolve();
		});
	});
}
