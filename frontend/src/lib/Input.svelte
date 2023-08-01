<script lang="ts">
	export let label: string = 'Input';
	export let value: string | number = '';
	export let id: string = 'input';

	export let type = 'text';

	export let maxlength: number = 255;

	export let min: number | null = null;
	export let max: number | null = null;

	let classes =
		'shadow appearance-none border rounded w-full py-2 px-3 ' +
		'text-gray-700 leading-tight focus:outline-none focus:shadow-outline';
</script>

<div class="mb-4">
	<label class="block text-gray-700 text-sm font-bold mb-2" for={id}>
		{label}
	</label>
	{#if type == 'text'}
		<input type="text" bind:value class={classes} {id} {maxlength} />
	{/if}
	{#if type == 'number'}
		<input type="number" bind:value class={classes} {id} min={min || null} max={max || null} />
	{/if}
</div>
