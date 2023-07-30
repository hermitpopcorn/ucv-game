<script lang="ts">
	import { onMount } from 'svelte';
	import { getGameState } from '$base/game';
	import { gameState, player } from '$base/stores';
	import Spinner from '$base/lib/Spinner.svelte';
	import type { Player } from '$base/types';

	let refreshingGameState = false;
	onMount(async () => {
		refreshingGameState = true;
		await getGameState();
		refreshingGameState = false;
	});
</script>

<section class="flex flex-1 flex-col justify-around items-center">
	{#if refreshingGameState || $gameState == null}
		<div
			class="fixed top-0 left-0 right-0 bottom-0 w-full h-screen z-50
			overflow-hidden bg-slate-600 bg-opacity-50 flex flex-col items-center justify-center"
		>
			<Spinner size={12} />
		</div>
	{/if}

	<div class="flex flex-col w-full box-border justify-center" style="flex: 0 0 80%">
		<section>
			{#if $gameState}
				{#if $gameState?.round == null}
					<h1 class="">Game has not started yet. Please wait!</h1>
				{/if}
			{/if}
		</section>
	</div>
	<div class="flex flex-col w-full box-border justify-center" style="flex: 0 0 20%">
		<aside>
			<h1 class="font-bold text-lg mb-4">Active Players</h1>
			<ul class="flex flex-wrap gap-4 justify-center">
				{#if $gameState?.players}
					{#each $gameState.players as player}
						<li class="flex flex-col items-center">
							<h3 class="text-sm">{player.name}</h3>
							<h4 class="text-xs">{player.points} P</h4>
						</li>
					{/each}
				{/if}
			</ul>
		</aside>
	</div>
</section>
