import { v4 as generateUuid } from 'uuid';
import { getWebsocketConnection, pushResponseStack } from '$base/game';
import { organizer as organizerStore } from '$base/stores';

import type { Choice, Organizer, Player, Round } from './types';

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
	return new Promise((resolve, reject) => {
		const socket = getWebsocketConnection();
		const responseId = generateUuid();
		socket.send(
			JSON.stringify({
				responseId,
				action: 'set-round',
				payload: round,
			}),
		);

		pushResponseStack(responseId, resolve, reject);
	});
}

export function togglePlayerCanVote(player: Player, newCanVoteStatus: boolean): Promise<void> {
	return new Promise((resolve, reject) => {
		const socket = getWebsocketConnection();
		const responseId = generateUuid();
		socket.send(
			JSON.stringify({
				responseId,
				action: 'set-player-can-vote',
				payload: {
					id: player.id,
					canVote: newCanVoteStatus,
				},
			}),
		);

		pushResponseStack(responseId, resolve, reject);
	});
}

export function toggleVoteIsLie(choice: Choice, newLieStatus: boolean): Promise<void> {
	return new Promise((resolve, reject) => {
		const socket = getWebsocketConnection();
		const responseId = generateUuid();
		socket.send(
			JSON.stringify({
				responseId,
				action: 'set-vote-is-lie',
				payload: {
					id: choice.id,
					lie: newLieStatus,
				},
			}),
		);

		pushResponseStack(responseId, resolve, reject);
	});
}

export function changePlayerPoint(player: Player, amount: number): Promise<void> {
	let points = (player.points ?? 0) + amount;
	if (points < 0) {
		points = 0;
	}

	return new Promise((resolve, reject) => {
		const socket = getWebsocketConnection();
		const responseId = generateUuid();
		socket.send(
			JSON.stringify({
				responseId,
				action: 'set-player-points',
				payload: {
					id: player.id,
					points,
				},
			}),
		);

		pushResponseStack(responseId, resolve, reject);
	});
}
