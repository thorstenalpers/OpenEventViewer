import { z } from 'zod';
import { isMockHost } from './client';

/**
 * The push half of the bridge: what the host says on its own, as opposed to what it answers. Each
 * payload is parsed against the contract for the same reason replies are — a host that drifts fails
 * here rather than three components later.
 */
export const events = {} as const satisfies Record<string, z.ZodType>;

export type EventName = keyof typeof events;
export type EventPayload<T extends EventName> = z.infer<(typeof events)[T]>;

/** Listens until the returned function is called. Against the mock host nothing ever arrives. */
export async function subscribe<T extends EventName>(
	name: T,
	handler: (payload: EventPayload<T>) => void
): Promise<() => void> {
	if (isMockHost()) return () => {};

	const { listen } = await import('@tauri-apps/api/event');
	return await listen<unknown>(name, (event) => {
		const parsed = (events[name] as z.ZodType).safeParse(event.payload);
		if (parsed.success) handler(parsed.data as EventPayload<T>);
	});
}
