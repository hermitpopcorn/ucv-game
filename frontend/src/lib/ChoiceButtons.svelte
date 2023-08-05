<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher();

	export let choiceA = '';
	export let choiceB = '';
	export let fixed = false;
	export let disabled = true;
	export let selected: 'a' | 'b' | undefined;
	export let interactable = false;

	let baseButtonClass = 'w-full text-white font-bold py-2 px-4 border-b-4 rounded ';
	let greenButtonClass = 'bg-green-600 border-green-800';
	let selectedGreenButtonClass = 'border-green-500 bg-green-400';
	let redButtonClass = 'bg-red-600 border-red-800';
	let selectedRedButtonClass = 'border-red-500 bg-red-400';
	let fixedButtonClass = 'bg-blue-500 border-blue-700';
	let unselectedButtonClass = 'bg-gray-300 border-gray-400';
	let disabledButtonClass = 'bg-gray-400 border-gray-500';

	function getButtonClass(
		which: 'a' | 'b',
		selectedChoice: 'a' | 'b' | undefined,
		isSelectionFixed: boolean,
		isDisabled: boolean,
	): string {
		if (isDisabled) {
			return baseButtonClass + disabledButtonClass;
		}

		if (which != selectedChoice && !isSelectionFixed) {
			if (which == 'a') {
				return baseButtonClass + greenButtonClass;
			} else if (which == 'b') {
				return baseButtonClass + redButtonClass;
			}
		}

		if (which == selectedChoice && !isSelectionFixed) {
			if (which == 'a') {
				return baseButtonClass + selectedGreenButtonClass;
			} else if (which == 'b') {
				return baseButtonClass + selectedRedButtonClass;
			}
		}

		if (isSelectionFixed) {
			if (which == selectedChoice) {
				return baseButtonClass + fixedButtonClass;
			} else {
				return baseButtonClass + unselectedButtonClass;
			}
		}

		return baseButtonClass;
	}

	$: buttonAClass = getButtonClass('a', selected, fixed, disabled);
	$: buttonBClass = getButtonClass('b', selected, fixed, disabled);

	function confirmChoice(choice: 'a' | 'b') {
		if (!interactable) {
			return;
		}

		if (fixed || disabled) {
			return;
		}

		if (selected == null || selected != choice) {
			selected = choice;
			return;
		}

		if (selected == choice) {
			dispatch('finalized', choice);
		}
	}
</script>

<div class="w-full">
	<div class="w-full h-32 flex flex-1 gap-4 mb-4">
		<button
			class={buttonAClass}
			on:click={() => {
				confirmChoice('a');
			}}
		>
			<div class="flex flex-col items-center justify-center">
				<p class="text-lg font-bold">A: {choiceA}</p>
				{#if selected == 'a' && fixed}
					<p class="text-sm">You chose this option.</p>
				{/if}
			</div>
		</button>
		<button
			class={buttonBClass}
			on:click={() => {
				confirmChoice('b');
			}}
		>
			<div class="flex flex-col items-center justify-center">
				<p class="text-lg font-bold">B: {choiceB}</p>
				{#if selected == 'b' && fixed}
					<p class="text-sm">You chose this option.</p>
				{/if}
			</div>
		</button>
	</div>
	{#if interactable}
		<div class="w-full text-center text-sm">
			{#if disabled}
				You are not allowed to vote on this round.
			{:else if selected == null && !fixed}
				Click one of the buttons twice to make your choice.
			{:else if selected != null && !fixed}
				Click the button one more time to finalize.
			{:else if selected != null && fixed}
				You've chosen to pick option {selected.toUpperCase()}!
			{/if}
		</div>
	{/if}
</div>
