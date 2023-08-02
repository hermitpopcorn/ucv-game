<script lang="ts">
	import { gameState } from '$base/stores';
	import type { Choice, Player } from '$base/types';

	type PlayerWithChoice = {
		player: Player;
		choice?: Choice;
	};

	let activePlayersData: Array<PlayerWithChoice> = [];
	gameState.subscribe((gs) => {
		let players = gs?.players;
		if (!players) {
			return [];
		}

		activePlayersData = [];

		for (let player of players) {
			let choices = gs?.choices;
			let choice = choices?.get(player.id);

			activePlayersData.push({
				player: player,
				choice,
			});
		}
	});
</script>

<aside class={$$restProps.class || ''}>
	<h1 class="font-bold text-lg mb-4">Active Players</h1>
	<ul class="flex flex-col flex-wrap gap-4 justify-center">
		{#if $gameState?.players}
			{#each activePlayersData as data (data.player.id)}
				<li class="flex flex-col items-center">
					<h3 class="text-sm">{data.player.name}</h3>
					<h4 class="text-xs">{data.player.points} P</h4>
					<button class="text-xs bg-blue-400 hover:bg-blue-600 text-white font-bold px-2 rounded">
						Can Vote
						{#if data.player.canVote}
							✔️
						{:else}
							❌
						{/if}
					</button>
					{#if $gameState.choices}
						<p>
							Vote:
							{#if data.choice}
								<strong>{data.choice.option.toUpperCase()}</strong>
								<button
									class="text-xs bg-blue-400 hover:bg-blue-600 text-white font-bold px-2 rounded"
								>
									Lie
									{#if data.choice.lie}
										✔️
									{:else}
										❌
									{/if}
								</button>
							{:else}
								-
							{/if}
						</p>
					{/if}
				</li>
			{/each}
		{/if}
	</ul>
</aside>
