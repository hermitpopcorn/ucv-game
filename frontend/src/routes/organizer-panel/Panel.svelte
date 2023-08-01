<script lang="ts">
	import Button from '$base/lib/Button.svelte';
	import Input from '$base/lib/Input.svelte';
	import { gameState } from '$base/stores';
	import type { Round, RoundState } from '$base/types';
	import PlayerList from './PlayerList.svelte';

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

	function updateRound() {
		// TODO
	}
</script>

<section class="flex flex-1 w-full">
	<div class="flex flex-col w-full box-border" style="flex: 0 0 80%">
		<article class="border-4 p-4">
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
					<Button class="w-full" on:click={updateRound}>Set</Button>
				</div>
			</form>
		</article>
	</div>
	<div class="flex flex-col w-full box-border items-center" style="flex: 0 0 20%">
		<PlayerList class="border-4 p-4" />
	</div>
</section>
