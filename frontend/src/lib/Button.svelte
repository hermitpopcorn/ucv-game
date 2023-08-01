<script lang="ts">
	let extraClasses = '';
	export { extraClasses as class };

	let defaultClasses =
		'bg-blue-500 hover:bg-blue-400 text-white font-bold py-2 px-4 border-b-4 border-blue-700 hover:border-blue-500 rounded';

	let classes = [defaultClasses, extraClasses].join(' ');
</script>

<button class={classes} on:click>
	<div class="flex items-center justify-center">
		<slot />
	</div>
</button>
