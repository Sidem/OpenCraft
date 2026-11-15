// The tab strip on the menu's Settings page: one group of settings shows at a time, so nothing needs a long scroll. Markup
// drives it (index.html): a `.settings-tab` button and a `.settings-group` with the same `data-group`. The chosen tab is
// remembered in this browser. To add a group: a button and a group there, and the panel that fills it in main.ts.

import './settings-tabs.css';

const KEY = 'opencraft.settingsTab';

export function wireSettingsTabs(page: HTMLElement): void {
  const tabs = [...page.querySelectorAll<HTMLButtonElement>('.settings-tab')];
  const groups = [...page.querySelectorAll<HTMLElement>('.settings-group')];
  if (!tabs.length) return;

  const show = (group: string, remember: boolean) => {
    for (const tab of tabs) {
      const on = tab.dataset.group === group;
      tab.classList.toggle('active', on);
      tab.setAttribute('aria-selected', String(on));
      tab.tabIndex = on ? 0 : -1;
    }
    for (const g of groups) g.classList.toggle('hidden', g.dataset.group !== group);
    page.closest('.menu-main')?.scrollTo(0, 0);
    if (!remember) return;
    try {
      localStorage.setItem(KEY, group);
    } catch {
      // Not remembered; the first tab shows next time.
    }
  };

  tabs.forEach((tab, i) => {
    tab.addEventListener('click', () => show(tab.dataset.group!, true));
    // Left and right arrows move along the strip, as in any tab list.
    tab.addEventListener('keydown', (e) => {
      const step = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
      if (!step) return;
      e.preventDefault();
      const next = tabs[(i + step + tabs.length) % tabs.length];
      next.focus();
      show(next.dataset.group!, true);
    });
  });

  let saved: string | null = null;
  try {
    saved = localStorage.getItem(KEY);
  } catch {
    // Storage unavailable: start on the first tab.
  }
  show(tabs.some((t) => t.dataset.group === saved) ? saved! : tabs[0].dataset.group!, false);
}
