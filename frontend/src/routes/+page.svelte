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

<section class="flex flex-col justify-center items-center" style="flex: 0.6">
	{#if page === 'login'}
		<div out:fade={{ duration: 90 }} in:fade={{ delay: 100 }}>
			<Login />
		</div>
	{/if}
	{#if page === 'game'}
		<div out:fade={{ duration: 90 }} in:fade={{ delay: 100 }}>
			<Game />
		</div>
	{/if}
</section>
