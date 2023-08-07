<script lang="ts">
	import { changePlayerPoint, togglePlayerCanVote, toggleVoteIsLie } from '$base/organizer';
	import { gameState } from '$base/stores';
	import type { Choice, Player } from '$base/types';
	import { toast } from '@zerodevx/svelte-toast';

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

	let working = false;
	async function toggleCanVote(player: Player) {
		if (working) {
			return;
		}
		working = true;

		const updatingToast = toast.push('Toggling ability to vote...', { initial: 0 });
		try {
			await togglePlayerCanVote(player, !player.canVote);
			toast.pop(updatingToast);
			toast.push('Player updated.', {
				classes: ['toast success'],
			});
		} catch {
			toast.pop(updatingToast);
			toast.push('Failed to update player.', {
				classes: ['toast failure'],
			});
		} finally {
			working = false;
		}
	}

	async function toggleLie(choice: Choice | undefined) {
		if (working) {
			return;
		}
		if (!choice) {
			return;
		}
		working = true;

		const updatingToast = toast.push('Toggling lie status...', { initial: 0 });

		try {
			await toggleVoteIsLie(choice, !choice.lie);
			toast.pop(updatingToast);
			toast.push('Vote updated.', {
				classes: ['toast success'],
			});
		} catch {
			toast.pop(updatingToast);
			toast.push('Failed to update vote.', {
				classes: ['toast success'],
			});
		} finally {
			working = false;
		}
	}

	let pointChangeAmount = 1;
	async function changePoint(player: Player, point: number) {
		if (working) {
			return;
		}
		working = true;

		const updatingToast = toast.push('Updating points...', { initial: 0 });
		try {
			await changePlayerPoint(player, point);
			toast.pop(updatingToast);
			toast.push('Player points updated.', {
				classes: ['toast success'],
			});
		} catch {
			toast.pop(updatingToast);
			toast.push('Failed to update player points.', {
				classes: ['toast failure'],
			});
		} finally {
			working = false;
		}
	}
</script>

<aside class={$$restProps.class || ''}>
	<h1 class="font-bold text-lg mb-4">Active Players</h1>
	<ul class="flex flex-wrap gap-8 justify-center">
		{#if $gameState?.players}
			{#each activePlayersData as data (data.player.id)}
				<li class="flex flex-col items-center border-2 p-2">
					<h3 class="text-sm">{data.player.name}</h3>
					<h4 class="text-sm">
						{data.player.points} P
						<input type="number" bind:value={pointChangeAmount} min="1" max="9" class="w-8" />
						<button
							class="text-xs bg-blue-400 hover:bg-blue-600 text-white font-bold px-2 rounded"
							on:click={() => changePoint(data.player, pointChangeAmount * 1)}>+</button
						>
						<button
							class="text-xs bg-blue-400 hover:bg-blue-600 text-white font-bold px-2 rounded"
							on:click={() => changePoint(data.player, pointChangeAmount * -1)}>-</button
						>
					</h4>
					<button
						class="text-xs bg-blue-400 hover:bg-blue-600 text-white font-bold px-2 rounded"
						on:click={() => toggleCanVote(data.player)}
					>
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
									on:click={() => toggleLie(data.choice)}
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
