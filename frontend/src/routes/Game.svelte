<script lang="ts">
	import { onMount } from 'svelte';
	import { getGameState } from '$base/game';
	import { gameState, player } from '$base/stores';
	import ChoiceButtons from '$base/lib/ChoiceButtons.svelte';
	import Spinner from '$base/lib/Spinner.svelte';
	import PlayerList from './PlayerList.svelte';

	let refreshingGameState = false;
	onMount(async () => {
		refreshingGameState = true;
		await getGameState();
		refreshingGameState = false;
	});

	function finalizeVote(e: CustomEvent) {
		let selected: 'a' | 'b' = e.detail;
	}
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
					<h1 class="text-lg">Game has not started yet. Please wait!</h1>
				{:else}
					<h1 class="text-xl font-bold text-center">
						Round {$gameState.round.number}-{$gameState.round.phase}
					</h1>
					{#if $gameState.round.state == 'standby'}
						<h2 class="text-lg text-center">Are you ready for the next round?</h2>
					{:else}
						<h2 class="text-4xl mb-6 text-center">{$gameState.round.question}</h2>

						{#if $gameState.round.state == 'show-choices' || $gameState.round.state == 'voting-time' || $gameState.round.state == 'voting-locked'}
							<ChoiceButtons
								choiceA={$gameState.round.choiceA}
								choiceB={$gameState.round.choiceB}
								on:finalized={finalizeVote}
							/>
						{/if}
					{/if}
				{/if}
			{/if}
		</section>
	</div>
	<div class="flex flex-col w-full box-border justify-center" style="flex: 0 0 20%">
		<PlayerList />
	</div>
</section>
