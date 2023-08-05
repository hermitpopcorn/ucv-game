<script lang="ts">
	import { SvelteToast } from '@zerodevx/svelte-toast';
	import './styles.css';
	import '$base/app.css';

	import { onMount } from 'svelte';
	import { browserEnv, player, websocketConnection } from '$base/stores';
	import { connect } from '$base/game';

	onMount(async () => {
		var queryDict: Map<string, string> = new Map();
		location.search
			.substring(1)
			.split('&')
			.forEach(function (item) {
				queryDict.set(item.split('=')[0], item.split('=')[1]);
			});
		if (queryDict.get('server')) {
			browserEnv.set({ server: queryDict.get('server')! });
		}

		connect();
	});
</script>

<div class="flex flex-col min-h-screen">
	<main class="flex flex-1 flex-col p-4 w-full max-w-5xl my-0 mx-auto box-border">
		<slot />
	</main>

	<footer class="flex flex-col justify-center items-center p-4">
		{#if $websocketConnection.state == 'connecting'}
			<p>Connecting...</p>
		{/if}
		{#if $websocketConnection.state == 'disconnected' || $websocketConnection.state == 'error'}
			<p>Disconnected from game server.</p>
		{/if}
		{#if $websocketConnection.state == 'connected'}
			{#if $player}
				<p>Playing as <strong>{$player.name}</strong>.</p>
			{:else}
				<p>Connected to game server.</p>
			{/if}
		{/if}
	</footer>

	<SvelteToast />
</div>
