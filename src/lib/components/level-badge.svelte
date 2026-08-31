<script lang="ts">
	import { Badge, type BadgeVariant } from '$lib/components/ui/badge';

	interface Props {
		level: number;
		label: string;
	}

	let { level, label }: Props = $props();

	// One look per tier: Critical filled red, Error outlined red, Warning amber, Information plain,
	// Verbose dimmed — so a column of badges reads as severity without reading a single label.
	const variant: BadgeVariant = $derived(
		level === 1 || level === 2 ? 'destructive' : level === 3 ? 'warning' : 'neutral'
	);

	const emphasis = $derived(
		level === 1
			? 'border-destructive bg-destructive text-destructive-foreground'
			: level === 5
				? 'opacity-60'
				: ''
	);
</script>

<Badge {variant} class={emphasis}>{label}</Badge>
