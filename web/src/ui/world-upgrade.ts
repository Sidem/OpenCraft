// Deposits-layer upgrade notice and control (`worldgen/layers.rs`, Action::UpgradeDeposits):
// - On the Home page: a one-time dismissible notice when the open world's deposits layer is older than the latest.
// - On the Worlds page: an "Upgrade deposits" card whenever the open world can safely upgrade (host/solo only).
// Dismissing the one-time notice is stored locally per (worldId, latestDepositsVersion) so a future deposits update
// (e.g. gas fields) notifies once again; the Worlds page button remains available until upgraded.

import './world-upgrade.css';
import { button, h } from './dom';
import {
  shouldShowUpgradeNotice,
  UPGRADE_BODY,
  UPGRADE_DONE,
  UPGRADE_NEW_WORLD,
  UPGRADE_TITLE,
  upgradeNoticeKey,
} from './world-upgrade-text';

/** Minimal engine slice needed by `WorldUpgrade`. */
export interface UpgradeGame {
  can_upgrade_deposits(): boolean;
  latest_deposits_version(): number;
  upgrade_deposits(): void;
}

export interface WorldUpgradeParts {
  game: UpgradeGame;
  worldId: string;
  /** Persists the upgraded world once the engine tick has applied the action. */
  save: () => Promise<void>;
  /** Reports status text in the Worlds panel (errors or completion). */
  onStatus?: (text: string) => void;
}

export class WorldUpgrade {
  /** One-time dismissible notice for the Home page. */
  readonly notice: HTMLElement;
  /** Persistent upgrade card for the Worlds page (hidden once up to date). */
  readonly card: HTMLElement;
  private readonly buttons: HTMLButtonElement[] = [];
  private busy = false;
  onStatus: (text: string) => void;

  constructor(private readonly parts: WorldUpgradeParts) {
    this.onStatus = parts.onStatus ?? (() => {});
    this.notice = this.build(true);
    this.card = this.build(false);
    this.refresh();
  }

  /** Re-syncs visibility with `game.can_upgrade_deposits()` and the one-time dismissal flag. */
  refresh(): void {
    const can = this.parts.game.can_upgrade_deposits();
    const dismissed = readDismissed(this.noticeKey());
    this.notice.classList.toggle('hidden', !shouldShowUpgradeNotice(can, dismissed));
    this.card.classList.toggle('hidden', !can);
    for (const b of this.buttons) {
      b.disabled = this.busy || !can;
      b.textContent = this.busy ? 'Upgrading...' : UPGRADE_TITLE;
    }
  }

  private noticeKey(): string {
    return upgradeNoticeKey(this.parts.worldId, this.parts.game.latest_deposits_version());
  }

  private build(dismissible: boolean): HTMLElement {
    const box = h('div', 'world-upgrade hidden');
    const text = h('div', 'world-upgrade-text');
    text.append(
      h('strong', '', UPGRADE_TITLE),
      h('p', 'world-upgrade-body', UPGRADE_BODY),
      h('p', 'world-upgrade-note', UPGRADE_NEW_WORLD),
    );
    const actions = h('div', 'world-upgrade-actions');
    const go = button('secondary-btn active', UPGRADE_TITLE, () => void this.trigger());
    this.buttons.push(go);
    actions.append(go);
    if (dismissible) {
      actions.append(
        button('secondary-btn', 'Dismiss', () => {
          writeDismissed(this.noticeKey());
          this.refresh();
        }),
      );
    }
    box.append(text, actions);
    return box;
  }

  private async trigger(): Promise<void> {
    if (this.busy || !this.parts.game.can_upgrade_deposits()) return;
    this.busy = true;
    this.refresh();
    try {
      this.parts.game.upgrade_deposits();
      // The action applies on the next core tick (solo: ~16 ms, co-op host: ~INPUT_DELAY ticks).
      for (let i = 0; i < 60 && this.parts.game.can_upgrade_deposits(); i++) {
        await new Promise((r) => setTimeout(r, 20));
      }
      if (!this.parts.game.can_upgrade_deposits()) {
        writeDismissed(this.noticeKey());
        await this.parts.save();
        this.onStatus(UPGRADE_DONE);
      }
    } catch (err: unknown) {
      this.onStatus(err instanceof Error ? err.message : String(err));
    } finally {
      this.busy = false;
      this.refresh();
    }
  }
}

function readDismissed(key: string): boolean {
  try {
    return localStorage.getItem(key) !== null;
  } catch {
    return false;
  }
}

function writeDismissed(key: string): void {
  try {
    localStorage.setItem(key, '1');
  } catch {
    // Storage unavailable; the notice hides for this session on refresh.
  }
}
