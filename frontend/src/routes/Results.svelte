<script lang="ts">
	import { gameState as gameStateStore } from '$base/stores';
	import type { Player } from '$base/types';

	let winners: Array<Player> = [];
	let uniqueOption: 'a' | 'b' | 'draw' | undefined;
	gameStateStore.subscribe((gs) => {
		winners = [];

		if (!gs?.choices) {
			return;
		}

		let countA = 0;
		let aPlayers: Array<number> = [];
		let countB = 0;
		let bPlayers: Array<number> = [];
		for (let [playerId, choice] of gs.choices) {
			if (choice.lie) {
				continue;
			}
			if (!gs.players.get(playerId)) {
				continue;
			}

			if (choice.option == 'a') {
				countA++;
				aPlayers.push(playerId);
			} else if (choice.option == 'b') {
				countB++;
				bPlayers.push(playerId);
			}
		}

		if (countA == 0 || countB == 0) {
			uniqueOption = 'draw';
			return;
		}

		let winnerPlayers: Array<number> = [];
		if (countA < countB) {
			uniqueOption = 'a';
			winnerPlayers = aPlayers;
		} else if (countB < countA) {
			uniqueOption = 'b';
			winnerPlayers = bPlayers;
		} else {
			uniqueOption = 'draw';
		}

		for (let playerId of winnerPlayers) {
			let player = gs.players.get(playerId);
			if (!player) {
				continue;
			}
			winners.push(player);
		}
	});
</script>

<article class={$$restProps.class || ''}>
	{#if uniqueOption == 'draw'}
		<h1 class="font-bold text-3xl mb-4">Draw</h1>
	{:else if uniqueOption == 'a' || uniqueOption == 'b'}
		<h1 class="font-bold text-3xl mb-4 underline">Round Winners</h1>
		<ul class="flex flex-row justify-center gap-4">
			{#each winners as player}
				<li class="p-4"><h2 class="font-bold text-2xl">{player.name}</h2></li>
			{/each}
		</ul>
	{/if}
</article>
