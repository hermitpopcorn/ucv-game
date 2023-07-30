<script lang="ts">
	export let label: string = 'Input';
	export let value: string = '';
	export let id: string = 'input';
	export let maxlength: number = 255;
</script>

<div class="mb-4">
	<label class="block text-gray-700 text-sm font-bold mb-2" for={id}>
		{label}
	</label>
	<input
		bind:value
		class="shadow appearance-none border rounded w-full py-2 px-3 text-gray-700 leading-tight focus:outline-none focus:shadow-outline"
		type="text"
		{id}
		{maxlength}
	/>
</div>
