// What the Home page says under its title (ui/menu.ts `greet`): a title for each situation and a few lead lines, one picked
// at random each time the page is greeted. The plain line is always first, so it still turns up. Pure, so
// web/test/home-text.test.mjs can check it. To add a line: a string in `leads`, written for that situation.

/** first: a new player; back: a saved world opened; fresh: a new world opened; paused: Escape during play. */
export type Greeting = 'first' | 'back' | 'fresh' | 'paused';

export const GREETINGS: Record<Greeting, { title: string; leads: string[] }> = {
  first: { title: 'Welcome to OpenCraft', leads: [''] },
  back: {
    title: 'Welcome back',
    leads: [
      'Your world is just as you left it.',
      'Chief engineer of a very small factory.',
      'No enemies here. The only thing coming for you is a backlog.',
      'Your world saved itself. You can stop checking.',
    ],
  },
  fresh: {
    title: 'A new world',
    leads: [
      'Fresh ground, waiting for its first factory.',
      "Fresh ground, no rules. Please don't build the whole thing out of one conveyor.",
    ],
  },
  paused: {
    title: 'Paused',
    leads: [
      'Take your time. Press Esc or Resume when you are ready.',
      'Take your time. Nothing here is on fire. Yet.',
      'Your world saved itself. You can stop checking.',
      'No enemies here. The only thing coming for you is a backlog.',
    ],
  },
};

/** The title and one lead line for `kind`; `random` is a number from 0 up to (not including) 1. */
export function greeting(kind: Greeting, random: number = Math.random()): [string, string] {
  const { title, leads } = GREETINGS[kind];
  return [title, leads[Math.min(leads.length - 1, Math.floor(random * leads.length))]];
}
