<script lang="ts">
	import { Badge, Button, ToggleButton, ToggleButtonGroup } from '$lib/components/common'
	import { addWhitespaceBeforeCapitals, capitalize } from '$lib/utils'
	import { faArrowRight, faPen, faUpload, faUser } from '@fortawesome/free-solid-svg-icons'
	import { fieldTypeToTsType } from '../../utils'
	import Tooltip from '$lib/components/Tooltip.svelte'
	import Popover from '$lib/components/Popover.svelte'

	import ConnectedInputEditor from './inputEditor/ConnectedInputEditor.svelte'
	import EvalInputEditor from './inputEditor/EvalInputEditor.svelte'
	import RowInputEditor from './inputEditor/RowInputEditor.svelte'
	import StaticInputEditor from './inputEditor/StaticInputEditor.svelte'
	import UploadInputEditor from './inputEditor/UploadInputEditor.svelte'
	import { getContext } from 'svelte'
	import type { AppViewerContext, RichConfiguration } from '../../types'
	import type { InputType } from '../../inputType'

	export let id: string
	export let componentInput: RichConfiguration
	export let key: string
	export let hasRows: boolean = false
	export let userInputEnabled: boolean = false
	export let shouldCapitalize: boolean = true
	export let resourceOnly = false
	export let tooltip: string | undefined = undefined
	export let onlyStatic: boolean = false
	export let fieldType: InputType
	export let subFieldType: InputType | undefined
	export let format: string | undefined
	export let selectOptions: string[] | undefined
	export let fileUpload: { accept: string; convertTo: string } | undefined = undefined
	export let placeholder: string | undefined

	const { connectingInput } = getContext<AppViewerContext>('AppViewerContext')

	$: if (componentInput == undefined) {
		//@ts-ignore
		componentInput = {
			type: 'static',
			value: undefined
		}
	}

	import { onMount } from 'svelte'
	import { ChevronDown, PenTool, User, Upload, ArrowRight, Pen } from 'lucide-svelte'
	let isOpen = false
	let selected = componentInput.type

	let items = [
		{ value: 'static', label: 'Static', icon: Pen },
		{ value: 'user', label: 'User Input', icon: User },
		{ value: 'upload', label: 'Upload', icon: Upload },
		{ value: 'connected', label: 'Connect', icon: ArrowRight }
	]

	function handleClick(item) {
		selected = item.value
		isOpen = false
	}

	onMount(() => {
		const handleClickOutside = (e) => {
			if (!e.target.closest('.dropdown')) {
				isOpen = false
			}
		}
		document.addEventListener('click', handleClickOutside)
		return () => document.removeEventListener('click', handleClickOutside)
	})
</script>

{#if !(resourceOnly && (fieldType !== 'object' || !format?.startsWith('resource-')))}
	<div class="flex flex-row justify-between gap-1 w-full items-center">
		<div class="flex justify-between items-end gap-1 w-32">
			<div class="flex gap-x-2 gap-y-1 flex-wrap justify-end items-center">
				<span class="text-xs font-normal truncate text-gray-600">
					{shouldCapitalize ? capitalize(addWhitespaceBeforeCapitals(key)) : key}
					{#if tooltip}
						<Tooltip>
							{tooltip}
						</Tooltip>
					{/if}
				</span>
			</div>
		</div>

		<div class="w-full">
			{#if componentInput?.type === 'connected'}
				<ConnectedInputEditor bind:componentInput />
			{:else if componentInput?.type === 'row'}
				<RowInputEditor bind:componentInput />
			{:else if componentInput?.type === 'static'}
				<StaticInputEditor
					{fieldType}
					{subFieldType}
					{selectOptions}
					{format}
					{placeholder}
					bind:componentInput
				/>
			{:else if componentInput?.type === 'eval'}
				<EvalInputEditor {hasRows} {id} bind:componentInput />
			{:else if componentInput?.type === 'upload'}
				<UploadInputEditor bind:componentInput />
			{:else if componentInput?.type === 'user'}
				<span class="text-2xs italic text-gray-600">Field's value is set by the user</span>
			{/if}
		</div>

		{#if !onlyStatic && componentInput?.type && componentInput.type != 'eval'}
			<div class="relative dropdown">
				<Button size="xs" btnClasses="!h-8" color="dark" on:click={() => (isOpen = !isOpen)}>
					<div class="relative" />
					{#if items.find((item) => item.value === selected)}
						<svelte:component
							this={items.find((item) => item.value === selected)?.icon ?? PenTool}
							class="w-4 h-4"
						/>
						<div class="absolute bottom-0.5 right-1">
							<ChevronDown class="w-3 h-3" />
						</div>
					{/if}
				</Button>
				{#if isOpen}
					<div class="absolute mt-1 w-full rounded-md bg-white shadow-lg z-10">
						<ul class="py-1 rounded-md text-gray-700">
							{#each items as item (item.value)}
								<li>
									<button
										class="block w-full text-left px-4 py-2 hover:bg-gray-100"
										on:click={() => handleClick(item)}
									>
										<svelte:component this={item.icon} class="w-4 h-4 inline" />
										{item.label}
									</button>
								</li>
							{/each}
						</ul>
					</div>
				{/if}
			</div>
		{/if}
	</div>
{/if}
