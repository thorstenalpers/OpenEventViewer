<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import LockIcon from '@lucide/svelte/icons/lock';
	import { i18n } from '$lib/i18n/index.svelte';

	export interface Step {
		id: string;
		label: string;
		/** True when the app knows the answer itself — then the box states a fact and is not a choice. */
		derived: boolean;
		done: boolean;
	}

	interface Props {
		steps: Step[];
		toggle: (id: string, done: boolean) => void;
	}

	let { steps, toggle }: Props = $props();

	const t = $derived(i18n.t);
	const done = $derived(steps.filter((step) => step.done).length);

	// The step that was just ticked, cleared by its own animation ending. Held per id rather than as
	// a boolean, so ticking a second box while the first is still celebrating starts a new one.
	let celebrating = $state<string | null>(null);

	function flip(step: Step) {
		if (step.derived) return;
		const next = !step.done;
		if (next) celebrating = step.id;
		toggle(step.id, next);
	}
</script>

<div class="flex flex-col gap-2">
	<div class="flex items-baseline justify-between">
		<span class="text-sm font-medium">{t.exam.checklist}</span>
		<span class="text-xs text-muted-foreground tabular-nums">{done} / {steps.length}</span>
	</div>

	<ul class="flex flex-col gap-1">
		{#each steps as step (step.id)}
			<li>
				<button
					type="button"
					disabled={step.derived}
					onclick={() => flip(step)}
					title={step.derived ? t.exam.derived : undefined}
					class="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-start text-sm transition-colors
						{step.derived ? 'cursor-default' : 'cursor-pointer hover:bg-muted/60'}"
				>
					<span
						class="flex size-5 shrink-0 items-center justify-center rounded-md border transition-colors
							{step.done ? 'border-success bg-success text-white' : 'border-input'}
							{celebrating === step.id ? 'celebrate' : ''}"
						onanimationend={() => (celebrating = null)}
					>
						{#if step.done}
							<CheckIcon class="size-3.5" />
						{/if}
					</span>
					<span class={step.done ? 'text-muted-foreground line-through' : ''}>{step.label}</span>
					{#if step.derived}
						<LockIcon class="ms-auto size-3 opacity-40" />
					{/if}
				</button>
			</li>
		{/each}
	</ul>
</div>

<style>
	/* Two beats rather than one: the box swells as it is filled and settles back, which reads as
	   "that landed" where a single grow reads as a hover state that forgot to end. */
	@keyframes celebrate {
		0% {
			transform: scale(1);
		}
		40% {
			transform: scale(1.35);
			box-shadow: 0 0 0 6px color-mix(in oklab, var(--success) 25%, transparent);
		}
		100% {
			transform: scale(1);
			box-shadow: 0 0 0 0 transparent;
		}
	}

	.celebrate {
		animation: celebrate 420ms ease-out;
	}

	/* Someone who asked the system for less motion gets the colour change and nothing that moves. */
	@media (prefers-reduced-motion: reduce) {
		.celebrate {
			animation: none;
		}
	}
</style>
