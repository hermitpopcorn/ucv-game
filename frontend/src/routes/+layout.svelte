<script lang="ts">
	import { SvelteToast } from '@zerodevx/svelte-toast';
	import './styles.css';
	import '$base/app.css';

	import { onMount } from 'svelte';
	import { player, websocketConnection } from '$base/stores';
	import { connect } from '$base/game';

	onMount(async () => {
		connect();
	});
</script>

<div class="flex flex-col min-h-screen">
	<main class="flex flex-1 flex-col p-4 w-full max-w-5xl my-0 mx-auto box-border">
		<slot />
	</main>

	<footer class="flex flex-col justify-center items-center p-4">
		{#if $websocketConnection == null}
			<p>Connecting...</p>
		{/if}
		{#if $websocketConnection == false}
			<p>Disconnected from game server.</p>
		{/if}
		{#if $websocketConnection}
			{#if $player}
				<p>Playing as <strong>{$player.name}</strong>.</p>
			{:else}
				<p>Connected to game server.</p>
			{/if}
		{/if}
	</footer>

	<SvelteToast />
</div>
