/* Desktop-only gate: the game needs a keyboard and a mouse, so phones and tablets get a notice instead of the
   game. `isTouchDevice` decides (a mobile user agent, or a touch screen as the main input, so touch laptops
   still play); `showDesktopOnly` covers the page. main.ts checks it before loading the engine. */

import './desktop-only.css';
import { h } from './dom';

export function isTouchDevice(): boolean {
  const nav = navigator as Navigator & { userAgentData?: { mobile?: boolean } };
  if (nav.userAgentData?.mobile) return true;
  if (/Android|iPhone|iPad|iPod|Mobile|Silk|Kindle/i.test(nav.userAgent)) return true;
  // iPadOS reports a Mac; a Mac with a touch screen is an iPad.
  if (/Macintosh/.test(nav.userAgent) && nav.maxTouchPoints > 1) return true;
  // The main pointer is a finger and nothing hovers: a tablet or phone in desktop mode.
  return matchMedia('(pointer: coarse) and (hover: none)').matches;
}

export function showDesktopOnly(): void {
  document.getElementById('menu')?.remove();
  document.getElementById('hud')?.remove();
  document.getElementById('game')?.remove();
  const box = h('div', 'desktop-only');
  box.append(
    h('h1', '', 'OpenCraft'),
    h('p', '', 'This game only works on a desktop or laptop computer, with a keyboard and a mouse.'),
    h('p', 'desktop-only-sub', 'Open this page on a computer to play.'),
  );
  document.body.append(box);
}
