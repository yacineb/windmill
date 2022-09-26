<script lang="ts">
	import Button from '$lib/components/common/button/Button.svelte'

	import type { IconDefinition } from '@fortawesome/free-brands-svg-icons'
	import Icon from 'svelte-awesome'

	export let disabled: boolean = false
	export let icon: IconDefinition
	export let label: string
	export let iconColor: string
</script>

<Button {disabled} on:click size="md" btnClasses="text-left h-12" color="light" variant="border">
	<Icon data={icon} class={`mr-2 ${iconColor} w-4`} scale={1.5} />
	<span class="text-sm text-left"> {label} </span>
</Button>
