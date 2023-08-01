<script lang="ts">
	import { gameState } from '$base/stores';
</script>

<aside class={$$restProps.class || ''}>
	<h1 class="font-bold text-lg mb-4">Active Players</h1>
	<ul class="flex flex-wrap gap-4 justify-center">
		{#if $gameState?.players}
			{#each $gameState.players as player}
				<li class="flex flex-col items-center">
					<h3 class="text-sm">{player.name}</h3>
					<h4 class="text-xs">{player.points} P</h4>
				</li>
			{/each}
		{/if}
	</ul>
</aside>
