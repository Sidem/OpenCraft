// The Worlds page when the browser refuses storage (no IndexedDB), so nothing can be saved or imported.
// Embedded in another site (itch.io runs the game in an iframe on itch.zone), the usual cause is the
// browser blocking third-party storage, which it allows once the game is the page itself: so it offers
// to open the game in its own tab. main.ts builds it into `#worlds` in place of the Worlds panel.

import { soloUrl } from '../save/session';
import { button, h } from './dom';

/** `notice` is what `openWorld` said about the unsaved world. */
export function noStorage(notice: string): HTMLElement {
  const el = h('div', '', notice);
  if (!embedded()) return el;
  el.textContent =
    'Your browser blocks storage for games shown inside another site, so this world won\'t be saved and worlds ' +
    "can't be imported here. Opened in its own tab, the game can save and import.";
  const row = h('p');
  row.append(button('secondary-btn active', 'Open in its own tab', () => window.open(soloUrl(), '_blank')));
  el.append(row);
  return el;
}

/** True inside a frame of another page. */
function embedded(): boolean {
  try {
    return window.top !== window.self;
  } catch {
    return true;
  }
}
