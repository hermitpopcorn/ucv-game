<script lang="ts">
	import { getGameState } from '$base/game';
	import Button from '$base/lib/Button.svelte';

	async function refresh() {
		await getGameState();
	}
</script>

<section>
	<p>TODO</p>
	<p><Button on:click={refresh}>Retrieve Game State</Button></p>
</section>
