<script lang="ts">
	import { Badge } from '$lib/components/ui/badge';
	import type { ExamTimeline } from '$lib/bridge/contract';
	import { i18n } from '$lib/i18n/index.svelte';

	interface Props {
		exams: ExamTimeline[];
	}

	let { exams }: Props = $props();

	const t = $derived(i18n.t);

	const DAY = 24 * 60 * 60 * 1000;

	function time(value: string): number {
		const parsed = Date.parse(value);
		return Number.isNaN(parsed) ? Date.now() : parsed;
	}

	/**
	 * The window the axis spans: from the oldest thing that happened to today.
	 *
	 * A week of padding on each side, because a bar that starts on the left edge and a marker on
	 * the right edge both read as "cut off" rather than as "this is where it began".
	 */
	const span = $derived.by(() => {
		const stamps = exams.flatMap((exam) => [
			time(exam.startedAt),
			...exam.passed.map(time),
			...(exam.lastStudiedAt ? [time(exam.lastStudiedAt)] : [])
		]);
		const now = Date.now();
		const first = stamps.length ? Math.min(...stamps, now) : now - 30 * DAY;
		const last = Math.max(now, ...stamps);
		// A single-day span would divide by zero below; a month is the smallest useful axis.
		const width = Math.max(last - first, 30 * DAY);
		return { from: first - width * 0.04, to: last + width * 0.04 };
	});

	function offset(value: string): number {
		const at = time(value);
		return ((at - span.from) / (span.to - span.from)) * 100;
	}

	/** Where an exam's bar ends: at the last time it was passed, or at today while it is open. */
	function endsAt(exam: ExamTimeline): number {
		const last = exam.passed.at(-1);
		return last ? offset(last) : today;
	}

	const today = $derived(((Date.now() - span.from) / (span.to - span.from)) * 100);

	/** Year labels along the axis, one per year the window touches. */
	const ticks = $derived.by(() => {
		const from = new Date(span.from).getUTCFullYear();
		const to = new Date(span.to).getUTCFullYear();
		return Array.from({ length: to - from + 1 }, (_, index) => from + index)
			.map((year) => ({ year, at: offset(`${year}-01-01T00:00:00Z`) }))
			.filter((tick) => tick.at >= 0 && tick.at <= 100);
	});

	function date(value: string): string {
		return new Date(value).toLocaleDateString(i18n.locale);
	}
</script>

{#if exams.length}
	<div class="flex flex-col gap-2">
		<!-- The axis is drawn once above the rows rather than per row, so the dates line up by
		     position instead of by everyone reading the same numbers off different scales. -->
		<div class="relative h-4 border-b text-[10px] text-muted-foreground">
			{#each ticks as tick (tick.year)}
				<span class="absolute -translate-x-1/2 tabular-nums" style:left={`${tick.at}%`}>
					{tick.year}
				</span>
			{/each}
		</div>

		<ul class="flex flex-col gap-1.5">
			{#each exams as exam (exam.binderId)}
				<li class="grid grid-cols-[10rem_1fr] items-center gap-3">
					<span class="flex min-w-0 items-center gap-2">
						<Badge variant="accent">{exam.certification}</Badge>
						<span class="truncate text-xs text-muted-foreground">{exam.title}</span>
					</span>

					<div class="relative h-6">
						<div class="absolute inset-x-0 top-1/2 h-px -translate-y-1/2 bg-border"></div>

						<!-- From the day the exam was created to the last thing that happened to it. -->
						<div
							class="absolute top-1/2 h-1.5 -translate-y-1/2 rounded-full bg-primary/25"
							style:left={`${offset(exam.startedAt)}%`}
							style:width={`${Math.max(endsAt(exam) - offset(exam.startedAt), 0.6)}%`}
						></div>

						<span
							class="absolute top-1/2 size-2.5 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-background bg-primary"
							style:left={`${offset(exam.startedAt)}%`}
							title={`${t.timeline.started} ${date(exam.startedAt)}`}
						></span>

						{#each exam.passed as passed (passed)}
							<span
								class="absolute top-1/2 size-3 -translate-x-1/2 -translate-y-1/2 rotate-45 border-2 border-background bg-success"
								style:left={`${offset(passed)}%`}
								title={`${t.timeline.passed} ${date(passed)}`}
							></span>
						{/each}
					</div>
				</li>
			{/each}
		</ul>

		<div class="flex flex-wrap items-center gap-4 text-xs text-muted-foreground">
			<span class="flex items-center gap-1.5">
				<span class="size-2.5 rounded-full bg-primary"></span>
				{t.timeline.started}
			</span>
			<span class="flex items-center gap-1.5">
				<span class="size-2.5 rotate-45 bg-success"></span>
				{t.timeline.passed}
			</span>
		</div>
	</div>
{:else}
	<p class="text-sm text-muted-foreground">{t.timeline.empty}</p>
{/if}
