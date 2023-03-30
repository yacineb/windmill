<script lang="ts">
	import type { SvelteComponent } from 'svelte';
	import { PostgresIcon, Mysql } from '../../icons';
	import type { SupportedLanguage } from '../../utils';
	import { BashIcon, GoIcon, PythonIcon, TypeScriptIcon, JavaScriptIcon } from './';

	export let lang: SupportedLanguage | 'pgsql' | 'mysql' | 'javascript';
	export let width = 30;
	export let height = 30;
	export let scale = 1;
	const langToComponent: Record<
		SupportedLanguage | 'pgsql' | 'mysql' | 'javascript',
		typeof SvelteComponent
	> = {
		go: GoIcon,
		python3: PythonIcon,
		deno: TypeScriptIcon,
		bash: BashIcon,
		pgsql: PostgresIcon,
		mysql: Mysql,
		javascript: JavaScriptIcon
	};
</script>

<svelte:component
	this={langToComponent[lang]}
	width={width * scale}
	height={height * scale}
	{...$$restProps}
/>
