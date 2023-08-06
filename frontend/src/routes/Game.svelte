<script lang="ts">
	import { onMount } from 'svelte';
	import { getGameState } from '$base/game';
	import { gameState as gameStateStore, player as playerStore } from '$base/stores';
	import { get } from 'svelte/store';
	import { setChoice } from '$base/player';
	import ChoiceButtons from '$base/lib/ChoiceButtons.svelte';
	import Spinner from '$base/lib/Spinner.svelte';
	import PlayerList from './PlayerList.svelte';
	import { toast } from '@zerodevx/svelte-toast';
	import Votes from './Votes.svelte';
	import Results from './Results.svelte';

	let refreshingGameState = false;
	onMount(async () => {
		refreshingGameState = true;
		await getGameState();
		refreshingGameState = false;
	});

	let voteSelected: 'a' | 'b' | undefined = undefined;
	let voteFixed = false;

	gameStateStore.subscribe((gs) => {
		let player = get(playerStore);
		if (!player) {
			return;
		}

		let myVote = gs?.choices?.get(player.id);
		if (!myVote) {
			voteSelected = undefined;
			voteFixed = false;
			return;
		}

		voteSelected = myVote.option;
		voteFixed = true;
	});

	async function finalizeVote(e: CustomEvent) {
		if (voteFixed) {
			return;
		}

		let selected: 'a' | 'b' = e.detail;
		try {
			await setChoice(selected);
		} catch {
			toast.push('Failed to send vote data to server.', {
				classes: ['toast failure'],
			});
			return;
		}
		voteFixed = true;
		voteSelected = selected;
	}
</script>

<section class="flex flex-1 flex-col justify-around items-center">
	{#if refreshingGameState || $gameStateStore == null}
		<div
			class="fixed top-0 left-0 right-0 bottom-0 w-full h-screen z-50
			overflow-hidden bg-slate-600 bg-opacity-50 flex flex-col items-center justify-center"
		>
			<Spinner size={12} />
		</div>
	{/if}

	<div class="flex flex-col w-full box-border justify-center items-center" style="flex: 0 0 80%">
		<section>
			{#if $gameStateStore}
				{#if $gameStateStore?.round == null}
					<h1 class="text-lg">Game has not started yet. Please wait!</h1>
				{:else}
					<h1 class="text-xl font-bold text-center">
						Round {$gameStateStore.round.number}-{$gameStateStore.round.phase}
					</h1>
					{#if $gameStateStore.round.state == 'standby'}
						<h2 class="text-lg text-center">Are you ready for the next round?</h2>
					{:else}
						<h2 class="text-4xl mb-6 text-center">{$gameStateStore.round.question}</h2>

						{#if $gameStateStore.round.state == 'show-choices' || $gameStateStore.round.state == 'voting-time' || $gameStateStore.round.state == 'voting-locked'}
							<div class="sm:min-w-[640px] sm:w-fit w-full">
								<ChoiceButtons
									choiceA={$gameStateStore.round.choiceA}
									choiceB={$gameStateStore.round.choiceB}
									fixed={voteFixed}
									disabled={!($playerStore?.canVote ?? false)}
									selected={voteSelected}
									interactable={$gameStateStore.round.state == 'voting-time'}
									on:finalized={finalizeVote}
								/>
							</div>
						{/if}
						{#if $gameStateStore.round.state == 'show-votes'}
							<div class="flex items-center justify-center">
								<Votes />
							</div>
						{/if}
						{#if $gameStateStore.round.state == 'show-results'}
							<div class="flex items-center justify-center">
								<Results />
							</div>
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
