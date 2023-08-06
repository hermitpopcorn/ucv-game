<script lang="ts">
	import { fade } from 'svelte/transition';
	import { player } from '$base/stores';
	import Login from './Login.svelte';
	import Game from './Game.svelte';

	type Pages = 'login' | 'game';
	let page: Pages = 'login';

	$: playerLoggedIn = $player !== null;
	$: page = ((): Pages => {
		if (!playerLoggedIn) {
			return 'login';
		}

		return 'game';
	})();
</script>

<svelte:head>
	<title>UCV</title>
	<meta name="description" content="Unique Constraint Violation game" />
</svelte:head>

<div class="flex flex-1 flex-col justify-center items-center">
	{#if page === 'login'}
		<div
			class="flex flex-1 w-full justify-center"
			out:fade={{ duration: 90 }}
			in:fade={{ delay: 100 }}
		>
			<Login />
		</div>
	{/if}
	{#if page === 'game'}
		<div
			class="flex flex-1 w-full justify-center"
			out:fade={{ duration: 90 }}
			in:fade={{ delay: 100 }}
		>
			<Game />
		</div>
	{/if}
</div>
