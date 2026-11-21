import assert from 'node:assert/strict';
import test from 'node:test';
import {
  shouldShowUpgradeNotice,
  UPGRADE_BODY,
  UPGRADE_NEW_WORLD,
  upgradeNoticeKey,
} from '../src/ui/world-upgrade-text.ts';

test('upgrade notice key scopes to world id and latest deposits version', () => {
  assert.notEqual(upgradeNoticeKey('w1', 8), upgradeNoticeKey('w2', 8));
  assert.notEqual(upgradeNoticeKey('w1', 8), upgradeNoticeKey('w1', 13));
});

test('one-time notice shows only when upgradeable and not dismissed', () => {
  assert.equal(shouldShowUpgradeNotice(true, false), true);
  assert.equal(shouldShowUpgradeNotice(true, true), false);
  assert.equal(shouldShowUpgradeNotice(false, false), false);
  assert.equal(shouldShowUpgradeNotice(false, true), false);
});

test('upgrade copy states untouched ground and new-world note', () => {
  assert.match(UPGRADE_BODY, /not touched/i);
  assert.match(UPGRADE_BODY, /base is unchanged/i);
  assert.match(UPGRADE_NEW_WORLD, /new world/i);
});
