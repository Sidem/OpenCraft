// The "ask before closing" setting and its rule, kept apart from the DOM so they can be tested (`web/test`). For
// players whose browser deletes all site data when it closes: with the setting on, closing a tab whose world changed
// since it was last exported (or since the player said "not now") is held back so an export can be offered
// (`ui/leave-prompt.ts`). The setting lives in localStorage, so a browser that wipes it forgets it too.

const STORAGE_KEY = 'opencraft.askBeforeClosing';

/** Off unless the player turned it on. */
export function loadAskBeforeClosing(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === '1';
  } catch {
    return false;
  }
}

export function saveAskBeforeClosing(on: boolean): void {
  try {
    localStorage.setItem(STORAGE_KEY, on ? '1' : '0');
  } catch {
    // Not persisted; it still applies until the page closes.
  }
}

/** Whether closing should be held back: the setting is on and the world has moved on (`playSeconds`) past the
 * point the player last exported or dismissed the prompt (`coveredUpTo`). */
export function holdsClosing(enabled: boolean, playSeconds: number, coveredUpTo: number): boolean {
  return enabled && playSeconds > coveredUpTo;
}
