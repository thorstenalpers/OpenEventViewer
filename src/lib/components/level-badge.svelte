<script lang="ts">
	import { Badge } from '$lib/components/ui/badge';

	interface Props {
		level: number;
		label: string;
	}

	let { level, label }: Props = $props();

	// Critical and Error both read as "something broke"; only the warning tier earns its own colour,
	// and Information must not compete with either for attention.
	const variant = $derived(
		level === 1 || level === 2
			? ('destructive' as const)
			: level === 3
				? ('accent' as const)
				: ('neutral' as const)
	);
</script>

<Badge {variant}>{label}</Badge>
