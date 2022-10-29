<script lang="ts">
	import { classNames } from '$lib/utils'
	import { getContext } from 'svelte'
	import type { TabsContext } from './Tabs.svelte'

	export let value: string

	const { selected, update } = getContext<TabsContext>('Tabs')
</script>

<div
	class={classNames(
		value === $selected
			? 'border-b-2 border-gray-600 text-gray-800 '
			: 'hover:border-b-2 hover:border-gray-300 text-gray-500',
		'py-1 px-4 cursor-pointer transition-all ease-linear font-medium text-sm',
		$$props.class
	)}
	on:click={() => update(value)}
	on:keypress={() => update(value)}
>
	<div class="my-1"> <slot /> </div>
</div>
