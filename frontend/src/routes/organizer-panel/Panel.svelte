<script lang="ts">
	import { onMount } from 'svelte';
	import { getGameState } from '$base/game';
	import Button from '$base/lib/Button.svelte';
	import Input from '$base/lib/Input.svelte';
	import { gameState } from '$base/stores';
	import type { Round, RoundState } from '$base/types';
	import { updateRound } from '$base/organizer';
	import PlayerList from './PlayerList.svelte';
	import VotesList from './VotesList.svelte';
	import Spinner from '$base/lib/Spinner.svelte';
	import { toast } from '@zerodevx/svelte-toast';

	let refreshingGameState = false;
	onMount(async () => {
		refreshingGameState = true;
		await getGameState();
		refreshingGameState = false;
	});

	let round = 1;
	let phase = 1;
	let state: RoundState = 'standby';
	let question = '';
	let choiceA = '';
	let choiceB = '';

	gameState.subscribe((newGameState) => {
		if (!newGameState?.round) {
			return;
		}

		round = newGameState.round.number;
		phase = newGameState.round.phase;
		state = newGameState.round.state;
		question = newGameState.round.question;
		choiceA = newGameState.round.choiceA;
		choiceB = newGameState.round.choiceB;
	});

	let updating = false;

	async function setRound() {
		if (updating) {
			return;
		}
		updating = true;

		const updatingToast = toast.push('Updating round data...', { initial: 0 });

		try {
			await updateRound({
				id: 0,
				number: round,
				phase: phase,
				state: state,
				question: question,
				choiceA: choiceA,
				choiceB: choiceB,
			});
			toast.pop(updatingToast);
			toast.push('Round updated.', {
				classes: ['toast success'],
			});
		} catch {
			toast.pop(updatingToast);
			toast.push('Round updating failed.', {
				classes: ['toast failure'],
			});
		} finally {
			updating = false;
		}
	}
</script>

<section class="flex flex-1 w-full">
	{#if refreshingGameState || $gameState == null}
		<div
			class="fixed top-0 left-0 right-0 bottom-0 w-full h-screen z-50
			overflow-hidden bg-slate-600 bg-opacity-50 flex flex-col items-center justify-center"
		>
			<Spinner size={12} />
		</div>
	{/if}
	<div class="flex flex-col w-full box-border" style="flex: 0 0 80%">
		<article class="border-4 p-4 mb-4">
			<h1 class="font-bold text-lg mb-4">Round Setup</h1>
			<form>
				<div class="flex w-full gap-4">
					<div class="basis-4/12">
						<Input type="number" bind:value={round} id="round-number" label="Round" />
					</div>
					<div class="basis-4/12">
						<Input type="number" bind:value={phase} id="phase-number" label="Phase" />
					</div>
					<div class="basis-4/12">
						<div class="mb-4">
							<label class="block text-gray-700 text-sm font-bold mb-2" for="state-select">
								State
							</label>
							<select
								bind:value={state}
								class="bg-white shadow border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
								id="state-select"
							>
								<option>standby</option>
								<option>show-question</option>
								<option>show-choices</option>
								<option>voting-time</option>
								<option>voting-locked</option>
								<option>show-votes</option>
								<option>defense</option>
								<option>show-results</option>
							</select>
						</div>
					</div>
				</div>
				<Input bind:value={question} id="question" label="Question" />
				<div class="flex w-full gap-4">
					<div class="grow">
						<Input bind:value={choiceA} id="choice-a" label="Choice A" />
					</div>
					<div class="grow">
						<Input bind:value={choiceB} id="choice-b" label="Choice B" />
					</div>
				</div>
				<div>
					<Button class="w-full" on:click={setRound}>
						{#if !updating}
							Set
						{:else}
							<Spinner color="red" size={6} />
						{/if}
					</Button>
				</div>
			</form>
		</article>
		<PlayerList class="border-4 p-4 mb-4" />
	</div>
	<div class="flex flex-col w-full box-border items-center" style="flex: 0 0 20%">
		<VotesList class="border-4 p-4 text-center" />
	</div>
</section>
