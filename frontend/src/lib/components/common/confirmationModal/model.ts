export type BeforeNavigate = (
	callback: (newNavigationState: {
		to: { url: URL } | null
		from: { url: URL } | null
		cancel: () => void
	}) => void
) => void
