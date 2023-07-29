<script lang="ts">
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
		<Login />
	{/if}
	{#if page === 'game'}
		<Game />
	{/if}
</section>
