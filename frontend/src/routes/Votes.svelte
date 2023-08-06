<script lang="ts">
	import { gameState as gameStateStore } from '$base/stores';
	import type { GameState, Player } from '$base/types';

	type PlayerChoice = { player: Player; lie: boolean; id: number };
	type PlayerChoices = {
		a: Array<PlayerChoice>;
		b: Array<PlayerChoice>;
	};

	function getChoices(gameState: GameState | null): PlayerChoices {
		let struct: PlayerChoices = { a: [], b: [] };
		if (!gameState) {
			return struct;
		}

		for (let player of gameState.players) {
			let choice = gameState.choices.get(player.id);
			if (!choice) {
				continue;
			}

			struct[choice.option].push({ player, lie: choice.lie, id: choice.id });
		}

		let sorter = (a: PlayerChoice, b: PlayerChoice) => {
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

	function countTruthsOnly(choices: Array<PlayerChoice>): number {
		let count = 0;
		for (let c of choices) {
			if (c.lie) {
				continue;
			}

			count++;
		}

		return count;
	}

	let choices: PlayerChoices = { a: [], b: [] };
	gameStateStore.subscribe((gs) => {
		choices = getChoices(gs);
	});
</script>

<table class="border-collapse text-center w-full table-fixed">
	<thead>
		<tr>
			<th class="border-b-2 border-r-2 border-slate-300 p-2">
				{#if $gameStateStore?.round}
					A: {$gameStateStore.round.choiceA ?? '???'}
				{:else}
					A
				{/if}
			</th>
			<th class="border-l-2 border-b-2 border-slate-300 p-2">
				{#if $gameStateStore?.round}
					B: {$gameStateStore.round.choiceB ?? '???'}
				{:else}
					B
				{/if}
			</th>
		</tr>
	</thead>
	<tbody>
		<tr>
			<td class="border-r-2 border-slate-300 p-2">
				<ul class="flex flex-col">
					{#each choices.a as c (c.player.id)}
						<li class={c.lie ? 'line-through text-gray-400' : ''}>{c.player.name}</li>
					{/each}
				</ul>
			</td>
			<td class="border-l-2 border-slate-300 p-2">
				<ul class="flex flex-col">
					{#each choices.b as c (c.player.id)}
						<li class={c.lie ? 'line-through text-gray-400' : ''}>{c.player.name}</li>
					{/each}
				</ul>
			</td>
		</tr>
	</tbody>
	<tfoot>
		<tr>
			<td class="border-r-2 border-t-2 border-slate-300 p-2">
				{countTruthsOnly(choices.a)}
			</td>
			<td class="border-l-2 border-t-2 border-slate-300 p-2">
				{countTruthsOnly(choices.b)}
			</td>
		</tr>
	</tfoot>
</table>
