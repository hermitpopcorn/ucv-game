import type { ActivePlayersMap, Choice, ChoiceMap, GameState, Player } from '$base/types';

export type PlayerChoice = { player: Player; lie: boolean; id: number };
export type PlayerChoices = {
	a: Array<PlayerChoice>;
	b: Array<PlayerChoice>;
};

export function getChoices(gameState: GameState | null): PlayerChoices {
	const struct: PlayerChoices = { a: [], b: [] };
	if (!gameState) {
		return struct;
	}

	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	for (const [_id, player] of gameState.players) {
		const choice = gameState.choices.get(player.id);
		if (!choice) {
			continue;
		}

		struct[choice.option].push({ player, lie: choice.lie, id: choice.id });
	}

	const sorter = (a: PlayerChoice, b: PlayerChoice) => {
		if (a.id < b.id) {
			return -1;
		}
		if (a.id > b.id) {
			return 1;
		}
		return 0;
	};

	struct.a.sort(sorter);
	struct.b.sort(sorter);

	return struct;
}

export function countTruthsOnly(choices: Array<PlayerChoice>): number {
	let count = 0;
	for (const c of choices) {
		if (c.lie) {
			continue;
		}

		count++;
	}

	return count;
}

export function convertPlayersObjectToMap(player: object): ActivePlayersMap {
	const map: Map<number, Player> = new Map();
	if (player) {
		for (const c of Object.entries(player)) {
			map.set(Number(c[0]), c[1]);
		}
	}

	return map;
}

export function convertChoicesObjectToMap(choices: object): ChoiceMap {
	const map: Map<number, Choice> = new Map();
	if (choices) {
		for (const c of Object.entries(choices)) {
			map.set(Number(c[0]), c[1]);
		}
	}

	return map;
}
