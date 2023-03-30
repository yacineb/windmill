// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface Platform {}
	}
}

declare namespace svelte.JSX {
	interface DOMAttributes<T> {
		onclick_outside?: CompositionEventHandler<T>;
	}
}

declare namespace svelteHTML {
	interface HTMLAttributes<T> {
		'on:click_outside'?: (event: any) => void;
	}
}

export {};
