<script lang="ts">
	import type { Schema } from '$lib/common'

	import { faCheck, faExclamationTriangle } from '@fortawesome/free-solid-svg-icons'
	import Icon from 'svelte-awesome'
	import SchemaForm from '../SchemaForm.svelte'

	let isValid: boolean = true

	export let args: Record<string, any> = {}
	export let schema: Schema
</script>

<div class="h-full overflow-auto">
	<div class="p-4">
		<div class="break-all relative font-sans">
			<div class="items-baseline text-xs text-gray-700 italic hidden md:block">
				<p>
					Move the focus outside of the text editor to recompute the input schema from main
					signature or press Ctrl/Cmd+S
				</p>
				<p class="mt-4">
					{#if isValid}
						<Icon data={faCheck} class="text-green-600 mr-1" scale={0.6} />
						The current preview input matches requirements defined in arguments
					{:else}
						<Icon data={faExclamationTriangle} class="text-yellow-500 mr-1" scale={0.6} />
						The current preview input doesn't match requirements defined in arguments
					{/if}
				</p>
			</div>
			<SchemaForm {schema} bind:args bind:isValid />
		</div>
	</div>
</div>
