<script lang="ts">
	import { organizer } from '$base/stores';
	import Login from './Login.svelte';
	import Panel from './Panel.svelte';

	type Pages = 'login' | 'panel';
	let page: Pages = 'login';

	$: organizerLoggedIn = $organizer !== null;
	$: page = ((): Pages => {
		if (!organizerLoggedIn) {
			return 'login';
		}

		return 'panel';
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
	{#if page === 'panel'}
		<Panel />
	{/if}
</section>
