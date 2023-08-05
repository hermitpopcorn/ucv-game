<script lang="ts">
	import { gameState as gameStateStore } from '$base/stores';
	import { get } from 'svelte/store';
	import type { Choice, GameState, Player } from '$base/types';

	type PlayerChoice = { player: Player; lie: boolean };
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

			struct[choice.option].push({ player, lie: choice.lie });
		}

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

	$: choices = getChoices(get(gameStateStore));
</script>

<aside class={$$restProps.class || ''}>
	<h1 class="font-bold text-lg mb-4">Votes</h1>
	<h2 class="font-bold">A ({countTruthsOnly(choices.a)})</h2>
	<ul class="flex flex-col flex-wrap gap-4 justify-center">
		{#each choices.a as c (c.player.id)}
			<li class={c.lie ? 'line-through' : ''}>{c.player.name}</li>
		{/each}
	</ul>
	<hr class="my-4 border-2" />
	<h2 class="font-bold">B ({countTruthsOnly(choices.b)})</h2>
	<ul class="flex flex-col flex-wrap gap-4 justify-center">
		{#each choices.b as c (c.player.id)}
			<li class={c.lie ? 'line-through' : ''}>{c.player.name}</li>
		{/each}
	</ul>
</aside>
