<script lang="ts">
	import { fade } from 'svelte/transition';
	import { player, websocketConnection } from '$base/stores';
	import Spinner from '$lib/Spinner.svelte';
	import Button from '$lib/Button.svelte';
	import { connect } from '$base/game';
	import { login as playerLogin } from '$base/player';
	import TextInput from '$base/lib/TextInput.svelte';

	let phase: 'connect' | 'login' = 'connect';
	let playerName = '';

	function retryConnect() {
		connect();
	}

	function play() {
		if ($player) {
			return;
		}

		phase = 'login';
	}

	let loggingIn = false;
	async function login() {
		if (loggingIn || !playerName) {
			return;
		}
		loggingIn = true;
		await playerLogin(playerName);
		loggingIn = false;
	}
</script>

<section class="flex flex-col items-center justify-center">
	<div style="flex: 0 0 25%" />
	<div style="flex: 1 0 75%">
		<article class="mb-8 flex flex-1 justify-center">
			<h1 class="text-9xl text-left">UCV</h1>
			<h2 class="text-xl self-center text-right">Unique Constraint Violation</h2>
		</article>

		<article class="flex flex-col items-center">
			{#if phase == 'connect'}
				<div class="h-12">
					{#if $websocketConnection.state === 'connected'}
						<div out:fade={{ duration: 90 }} in:fade={{ delay: 100 }}>
							<Button on:click={play}>Play</Button>
						</div>
					{:else if $websocketConnection.state === 'connecting'}
						<div out:fade={{ duration: 90 }} in:fade={{ delay: 100 }}>
							<Spinner size={12} />
						</div>
					{:else if $websocketConnection.state === 'error' || $websocketConnection.state === 'disconnected'}
						<div out:fade={{ duration: 90 }} in:fade={{ delay: 100 }}>
							<Button on:click={retryConnect}>Connect</Button>
						</div>
					{/if}
				</div>
			{/if}
			{#if phase == 'login'}
				<div class="flex flex-col" out:fade={{ duration: 90 }} in:fade={{ delay: 100 }}>
					<TextInput bind:value={playerName} id="player-name" label="Player Name" />
					<Button on:click={login}>
						{#if !loggingIn}
							Start
						{:else}
							<Spinner color="red" size={6} />
						{/if}
					</Button>
				</div>
			{/if}
		</article>
	</div>
</section>
