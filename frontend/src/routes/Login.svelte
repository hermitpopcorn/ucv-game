<script lang="ts">
	import { player, websocketConnection } from '$base/stores';
	import Spinner from '$lib/Spinner.svelte';
	import Button from '$lib/Button.svelte';
	import { connect } from '$base/game';
	import { login as playerLogin } from '$base/player';
	import TextInput from '$base/lib/TextInput.svelte';
	import { toast } from '@zerodevx/svelte-toast';

	let phase: 'connect' | 'login' = 'connect';
	let playerName = '';

	function retryConnect() {
		connect();
	}

	function play() {
		if ($player) {
			toast.push('Alright lets go!', { classes: ['toast'] });
			return;
		}

		phase = 'login';
	}

	let loggingIn = false;
	async function login() {
		if (loggingIn) {
			return;
		}
		loggingIn = true;
		await playerLogin(playerName);
		loggingIn = false;
	}
</script>

<section>
	<div class="mb-8 flex flex-1 justify-center">
		<h1 class="text-9xl text-left">UCV</h1>
		<h2 class="text-xl self-center text-right">Unique Constraint Violation</h2>
	</div>

	<div class="flex flex-col items-center">
		{#if phase == 'connect'}
			<div class="h-12">
				{#if $websocketConnection.state === 'connected'}
					<Button on:click={play}>Play</Button>
				{:else if $websocketConnection.state === 'connecting'}
					<Spinner size={12} />
				{:else if $websocketConnection.state === 'error' || $websocketConnection.state === 'disconnected'}
					<Button on:click={retryConnect}>Connect</Button>
				{/if}
			</div>
		{/if}
		{#if phase == 'login'}
			<div class="flex flex-col">
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
	</div>
</section>
