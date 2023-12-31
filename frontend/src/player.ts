import { v4 as generateUuid } from 'uuid';
import { getWebsocketConnection, pushResponseStack } from './game';
import { player as playerStore } from '$base/stores';
import { get } from 'svelte/store';
import type { Player } from '$base/types';

export function setPlayer(player: Player) {
	playerStore.set(player);
}

export function setPlayerIfSelf(player: Player) {
	if (!get(playerStore)) {
		return;
	}
	if (get(playerStore)?.id !== player.id) {
		return;
	}

	setPlayer(player);
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

export function setChoice(choiceOption: string): Promise<void> {
	return new Promise((resolve, reject) => {
		const socket = getWebsocketConnection();
		const responseId = generateUuid();
		socket.send(
			JSON.stringify({
				responseId,
				action: 'set-choice',
				payload: choiceOption,
			}),
		);

		pushResponseStack(responseId, resolve, reject);
	});
}
