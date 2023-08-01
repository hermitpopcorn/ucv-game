<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher();

	export let choiceA = '';
	export let choiceB = '';

	let baseButtonClass = 'w-full text-white font-bold py-2 px-4 border-b-4 rounded';
	let greenButtonClass = 'bg-green-600 border-green-800';
	let selectedGreenButtonClass = 'border-green-500 bg-green-400';
	let redButtonClass = 'bg-red-600 border-red-800';
	let selectedRedButtonClass = 'border-red-500 bg-red-400';
	let fixedButtonClass = 'bg-blue-500 border-blue-700';
	let unselectedButtonClass = 'bg-gray-300 border-gray-400';

	function getButtonClass(
		which: 'a' | 'b',
		selectedChoice: 'a' | 'b' | null,
		isSelectionFixed: boolean,
	): string {
		if (which != selectedChoice && !isSelectionFixed) {
			if (which == 'a') {
				return baseButtonClass + ' ' + greenButtonClass;
			} else if (which == 'b') {
				return baseButtonClass + ' ' + redButtonClass;
			}
		}

		if (which == selectedChoice && !isSelectionFixed) {
			if (which == 'a') {
				return baseButtonClass + ' ' + selectedGreenButtonClass;
			} else if (which == 'b') {
				return baseButtonClass + ' ' + selectedRedButtonClass;
			}
		}

		if (isSelectionFixed) {
			if (which == selectedChoice) {
				return baseButtonClass + ' ' + fixedButtonClass;
			} else {
				return baseButtonClass + ' ' + unselectedButtonClass;
			}
		}

		return baseButtonClass;
	}

	$: buttonAClass = getButtonClass('a', selected, selectionFixed);
	$: buttonBClass = getButtonClass('b', selected, selectionFixed);

	let selected: null | 'a' | 'b' = null;
	let selectionFixed = false;
	function confirmChoice(choice: 'a' | 'b') {
		if (selectionFixed) {
			return;
		}

		if (selected == null || selected != choice) {
			selected = choice;
			return;
		}

		if (selected == choice) {
			selectionFixed = true;

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
				{#if selected == 'a' && selectionFixed}
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
				{#if selected == 'b' && selectionFixed}
					<p class="text-sm">You chose this option.</p>
				{/if}
			</div>
		</button>
	</div>
	<div class="w-full text-center text-sm">
		{#if selected == null && !selectionFixed}
			Click one of the buttons twice to make your choice.
		{:else if selected != null && !selectionFixed}
			Click the button one more time to finalize.
		{:else if selected != null && selectionFixed}
			You've chosen to pick option {selected.toUpperCase()}!
		{/if}
	</div>
</div>
