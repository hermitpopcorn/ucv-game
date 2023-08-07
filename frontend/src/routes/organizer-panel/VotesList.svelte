<script lang="ts">
	import { countTruthsOnly, getChoices, type PlayerChoices } from '$base/dataprocessor';
	import { gameState as gameStateStore } from '$base/stores';

	let choices: PlayerChoices = { a: [], b: [] };
	gameStateStore.subscribe((gs) => {
		choices = getChoices(gs);
	});
</script>

<aside class={$$restProps.class || ''}>
	<h1 class="font-bold text-lg mb-4">Votes</h1>
	<h2 class="font-bold">A ({countTruthsOnly(choices.a)})</h2>
	<ul class="flex flex-col flex-wrap gap-4 justify-center">
		{#each choices.a as c (c.player.id)}
			<li class={c.lie ? 'line-through' : ''}>{c.player.name}</li>
		{/each}
	</ul>
	<hr class="my-4 border-2" />
	<h2 class="font-bold">B ({countTruthsOnly(choices.b)})</h2>
	<ul class="flex flex-col flex-wrap gap-4 justify-center">
		{#each choices.b as c (c.player.id)}
			<li class={c.lie ? 'line-through' : ''}>{c.player.name}</li>
		{/each}
	</ul>
</aside>
