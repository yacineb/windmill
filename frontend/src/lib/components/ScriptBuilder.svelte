<script lang="ts">
	import { ScriptService, type Script } from '$lib/gen'

	import { goto } from '$app/navigation'
	import { page } from '$app/stores'
	import { inferArgs } from '$lib/infer'
	import {
		DENO_INIT_CODE,
		DENO_INIT_CODE_TRIGGER,
		initialCode,
		POSTGRES_INIT_CODE
	} from '$lib/script_helpers'
	import { workspaceStore } from '$lib/stores'
	import { emptySchema, encodeState, sendUserToast, setQueryWithoutLoad } from '$lib/utils'
	import { Breadcrumb, BreadcrumbItem } from 'flowbite-svelte'
	import { onDestroy } from 'svelte'
	import SvelteMarkdown from 'svelte-markdown'
	import Path from './Path.svelte'
	import RadioButton from './RadioButton.svelte'
	import Required from './Required.svelte'
	import ScriptEditor from './ScriptEditor.svelte'
	import ScriptSchema from './ScriptSchema.svelte'
	import CenteredPage from './CenteredPage.svelte'

	let editor: ScriptEditor
	let scriptSchema: ScriptSchema

	export let script: Script
	export let initialPath: string = ''
	export let template: 'pgsql' | undefined = undefined

	let pathError = ''

	$: setQueryWithoutLoad($page.url, 'state', encodeState(script))
	$: step = Number($page.url.searchParams.get('step')) || 1
	$: {
		if (script.language == 'python3') {
			script.is_trigger = false
		}
	}

	if (script.content == '') {
		initContent(script.language, template)
	}

	function initContent(language: 'deno' | 'python3', template: 'pgsql' | undefined) {
		script.content = initialCode(
			language,
			template == 'pgsql' ? 'pgsql' : script.is_trigger ? 'trigger' : undefined
		)
	}

	async function editScript(): Promise<void> {
		try {
			const newHash = await ScriptService.createScript({
				workspace: $workspaceStore!,
				requestBody: {
					path: script.path,
					summary: script.summary,
					description: script.description ?? '',
					content: script.content,
					parent_hash: script.hash != '' ? script.hash : undefined,
					schema: script.schema,
					is_template: script.is_template,
					language: script.language,
					is_trigger: script.is_trigger
				}
			})
			sendUserToast(`Success! New script version created with hash ${newHash}`)
			goto(`/scripts/get/${newHash}`)
		} catch (error) {
			sendUserToast(`Impossible to save the script: ${error.body}`, true)
		}
	}

	export function setCode(script: Script) {
		editor?.getEditor().setCode(script.content)

		if (scriptSchema) {
			if (script.schema) {
				scriptSchema.setSchema(script.schema)
			} else {
				scriptSchema.setSchema(emptySchema())
			}
		}
	}

	async function inferSchema() {
		await inferArgs(script.language, script.content, script.schema)
	}

	async function changeStep(step: number) {
		if (step == 3) {
			script.content = editor?.getEditor().getCode() ?? script.content
			await inferSchema()
			script.schema = script.schema
		}
		goto(`?step=${step}`)
	}

	onDestroy(() => {
		editor?.$destroy()
	})
</script>

<div class="flex flex-col h-screen">
	<!-- Nav between steps-->
	<div class="flex flex-col w-full px-4 py-1 border-b shadow-sm bg-gray-900">
		<div class="justify-between flex flex-row drop-shadow-sm w-full">
			<nav class="flex text-white  " aria-label="Breadcrumb">
				<ol class="inline-flex items-center space-x-1 md:space-x-3">
					<li class="inline-flex items-center">
						<a href="#" class="inline-flex items-center text-xs font-medium text-white ">
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="w-6 h-6"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75"
								/>
							</svg>

							Metadata
						</a>
					</li>
					<li>
						<div class="flex items-center ">
							<svg
								xmlns="http://www.w3.org/2000/svg"
								fill="none"
								viewBox="0 0 24 24"
								stroke-width="1.5"
								stroke="currentColor"
								class="w-6 h-6"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									d="M14.25 9.75L16.5 12l-2.25 2.25m-4.5 0L7.5 12l2.25-2.25M6 20.25h12A2.25 2.25 0 0020.25 18V6A2.25 2.25 0 0018 3.75H6A2.25 2.25 0 003.75 6v12A2.25 2.25 0 006 20.25z"
								/>
							</svg>

							<a href="#" class="ml-1 text-xs font-medium text-white md:ml-2">Code Editor</a>
						</div>
					</li>
					<li aria-current="page">
						<div class="flex items-center">
							<span
								class="ml-1 text-xs font-medium text-gray-900 md:ml-2 flex items-center bg-slate-200 p-1 rounded-md"
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									fill="none"
									viewBox="0 0 24 24"
									stroke-width="1.5"
									stroke="currentColor"
									class="w-6 h-6 mr-2"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10"
									/>
								</svg>
								UI Customisation
							</span>
						</div>
					</li>
				</ol>
			</nav>
			<div class="flex flex-row-reverse ml-2">
				{#if step != 3}
					<button
						disabled={step == 1 && pathError != ''}
						class="default-button px-6 max-h-8"
						on:click={() => {
							changeStep(step + 1)
						}}
					>
						Next
					</button>
				{:else}
					<button class="default-button px-6 self-end" on:click={editScript}>Save</button>
				{/if}

				{#if step == 2}
					<button
						class="default-button-secondary px-6 max-h-8 mr-2"
						on:click={async () => {
							await inferSchema()
							editScript()
						}}
					>
						Save (commit)
					</button>
				{/if}
			</div>
		</div>
	</div>

	<!-- metadata -->
	{#if step === 1}
		<CenteredPage>
			<div class="space-y-6">
				<Path
					bind:error={pathError}
					bind:path={script.path}
					{initialPath}
					on:enter={() => changeStep(2)}
					namePlaceholder="my_script"
					kind="script"
				>
					<div slot="ownerToolkit">
						Script permissions depend on their path. Select the group
						<span class="font-mono"> all </span>
						to share your script, and <span class="font-mono">user</span> to keep it private.
						<a href="https://docs.windmill.dev/docs/reference/namespaces">docs</a>
					</div>
				</Path>
				<h3 class="text-gray-700 border-b">Language</h3>
				<div class="max-w-md">
					<RadioButton
						label="Language"
						options={[
							['Typescript (Deno)', 'deno'],
							['Python 3.10', 'python3']
						]}
						on:change={(e) => initContent(e.detail, template)}
						bind:value={script.language}
					/>
				</div>
				{#if script.language == 'deno'}
					<h4 class="text-gray-700  border-b">Template</h4>

					<div class="max-w-md">
						<RadioButton
							label="Template"
							options={[
								['None', undefined],
								['PostgreSQL', 'pgsql']
							]}
							on:change={(e) => initContent(script.language, e.detail)}
							bind:value={template}
						/>
					</div>
				{/if}

				<h3 class="text-gray-700 pb-1 border-b">Metadata</h3>

				<label class="block ">
					<span class="text-gray-700">Summary <Required required={false} /></span>
					<textarea
						bind:value={script.summary}
						class="
					mt-1
					block
					w-full
					rounded-md
					border-gray-300
					shadow-sm
					focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50
					"
						placeholder="A very short summary of the script displayed when the script is listed"
						rows="1"
					/>
				</label>
				<label class="block" for="inp">
					<span class="text-gray-700"
						>Description<Required required={false} detail="accept markdown formatting" />
						<textarea
							id="inp"
							bind:value={script.description}
							class="
					mt-1
					block
					w-full
					rounded-md
					border-gray-300
					shadow-sm
					focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50
					"
							placeholder="A description to help users understand what this script does and how to use it."
							rows="3"
						/>
					</span>
				</label>

				<label class="block">
					<span class="text-gray-700 mr-2">Save as template</span>
					<input type="checkbox" bind:checked={script.is_template} />
				</label>

				<label class="block">
					<span class="text-gray-700 mr-2">Save as trigger script</span>
					<input
						disabled={script.language == 'python3'}
						type="checkbox"
						bind:checked={script.is_trigger}
						on:change={() => {
							if (
								script.content == DENO_INIT_CODE ||
								script.content == DENO_INIT_CODE_TRIGGER ||
								script.content == POSTGRES_INIT_CODE
							) {
								initContent(script.language, template)
							}
						}}
					/>
				</label>

				<div>
					<h3 class="text-gray-700 ">Description rendered</h3>
					<div
						class="prose mt-5 text-xs shadow-inner shadow-blue p-4 overflow-auto"
						style="max-height: 200px;"
					>
						<SvelteMarkdown source={script.description ?? ''} />
					</div>
				</div>
			</div>
		</CenteredPage>
	{:else if step === 2}
		<ScriptEditor
			bind:this={editor}
			bind:schema={script.schema}
			path={script.path}
			bind:code={script.content}
			lang={script.language}
		/>
	{:else if step === 3}
		<CenteredPage>
			<ScriptSchema
				bind:summary={script.summary}
				bind:description={script.description}
				bind:schema={script.schema}
			/>
		</CenteredPage>
	{/if}
</div>
