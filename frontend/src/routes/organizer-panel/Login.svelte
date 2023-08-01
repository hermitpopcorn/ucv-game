<script lang="ts">
	import { organizer, websocketConnection } from '$base/stores';
	import Spinner from '$lib/Spinner.svelte';
	import Button from '$lib/Button.svelte';
	import { connect } from '$base/game';
	import { login as organizerLogin } from '$base/organizer';
	import Input from '$base/lib/Input.svelte';

	let phase: 'connect' | 'login' = 'connect';
	let password = '';

	function retryConnect() {
		connect();
	}

	function play() {
		if ($organizer) {
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
		await organizerLogin(password);
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
					<Button on:click={play}>Organize</Button>
				{:else if $websocketConnection.state === 'connecting'}
					<Spinner size={12} />
				{:else if $websocketConnection.state === 'error' || $websocketConnection.state === 'disconnected'}
					<Button on:click={retryConnect}>Connect</Button>
				{/if}
			</div>
		{/if}
		{#if phase == 'login'}
			<div class="flex flex-col">
				<Input bind:value={password} id="organizer-password" label="Password" />
				<Button on:click={login}>
					{#if !loggingIn}
						Access
					{:else}
						<Spinner color="red" size={6} />
					{/if}
				</Button>
			</div>
		{/if}
	</div>
</section>
