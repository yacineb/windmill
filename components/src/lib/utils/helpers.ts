export function classNames(...classes: Array<string | undefined>): string {
	return classes.filter(Boolean).join(' ');
}

export function clickOutside(node: Node): { destroy(): void } {
	const handleClick = (event: MouseEvent) => {
		if (node && !node.contains(<HTMLElement>event.target) && !event.defaultPrevented) {
			node.dispatchEvent(new CustomEvent<MouseEvent>('click_outside', { detail: event }));
		}
	};

	document.addEventListener('click', handleClick, true);

	return {
		destroy() {
			document.removeEventListener('click', handleClick, true);
		}
	};
}
