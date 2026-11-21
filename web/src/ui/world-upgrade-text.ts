// Pure copy and one-time notice helpers for upgrading a world's deposits layer (ui/world-upgrade.ts).
// Kept free of DOM and CSS so `web/test/world-upgrade.test.mjs` runs it directly under Node.

export const UPGRADE_TITLE = 'Upgrade deposits';
export const UPGRADE_BODY = 'New ores and gas fields in ground you have not touched; your base is unchanged.';
export const UPGRADE_NEW_WORLD = 'A new world has more changes: terrain, rivers, trees.';
export const UPGRADE_DONE = 'Deposits upgraded. Ground you have not touched now uses the latest deposits.';

/** Storage key for the one-time Home notice: scoped to this world and the latest deposits version. */
export function upgradeNoticeKey(worldId: string, latestDeposits: number): string {
  return `opencraft.depositsNotice.${worldId}.${latestDeposits}`;
}

/** Whether the one-time Home notice should show. */
export function shouldShowUpgradeNotice(canUpgrade: boolean, dismissed: boolean): boolean {
  return canUpgrade && !dismissed;
}
