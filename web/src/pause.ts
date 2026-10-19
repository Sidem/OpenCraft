// Playing and pausing: the menu's Play button, pointer lock, Escape, panels closing back to play, leaving the tab
// and a lost graphics context. Pausing shows the world's line on the pause screen and saves; leaving play stops the
// player's movement and hands. To add a screen that frees the mouse: list it in `closers` (main.ts) so closing it plays on.
import type { SoundSystem } from './audio/sound';
import type { Input } from './input';
import { clock } from './render/sky';
import type { Opened, Session } from './save/session';
import type { InventoryPanel } from './ui/inventory';
import type { Menu } from './ui/menu';
import type { SoundLab } from './ui/sound-lab';
import { SoundPanel } from './ui/sound-panel';
import type { Game } from './wasm/engine';

export interface PauseParts {
  game: Game;
  opened: Opened;
  viewRadius: number;
  menu: Menu;
  canvas: HTMLCanvasElement;
  input: Input;
  sound: SoundSystem;
  soundLab: SoundLab;
  session: Session | null;
  inventory: InventoryPanel;
  /** The other screens that free the mouse; closing one plays on or shows the resume hint. */
  closers: { onClose: (resume: boolean) => void }[];
  panelOpen: () => boolean;
}

/** Wires playing and pausing; returns `pause`, which shows the pause screen. */
export function wirePause(parts: PauseParts): () => void {
  const { game, opened, viewRadius, menu, canvas, input, sound, soundLab, session, inventory, closers, panelOpen } = parts;
  const { meta } = opened;
  const play = menu.play;
  // Worlds made before Milestone 4 keep their first terrain (generator version 1).
  const terrain = game.worldgen_version() === 1 ? ' · classic terrain' : '';
  const range = viewRadius === 8 ? '' : ` · render distance ${viewRadius}`;
  const showWorldInfo = () => {
    const time = `Day ${game.day_number()}, ${clock(game.time_of_day())}`;
    menu.world(meta.name, `${time} · seed ${meta.seed}${terrain}${range}`);
  };
  showWorldInfo();
  menu.opened(opened.restored);
  if (opened.restored) play.textContent = 'Continue';
  document.getElementById('sound-panel')!.append(new SoundPanel(sound).el);
  document.getElementById('open-sound-lab')!.addEventListener('click', () => {
    sound.unlock();
    soundLab.open(sound.lastMaterial);
  });
  // Audio can only start inside a user gesture, so the same click that captures the mouse unlocks it.
  const start = () => {
    sound.unlock();
    void input.lock();
  };
  play.addEventListener('click', start);
  canvas.addEventListener('click', () => {
    if (!input.locked && !play.disabled && !soundLab.isOpen && !panelOpen()) start();
  });
  const resumeHint = document.getElementById('resume-hint')!;
  const pause = () => {
    showWorldInfo();
    resumeHint.classList.add('hidden');
    menu.pause();
  };
  // E or a click outside goes straight back to play (both are gestures that allow re-locking). Escape
  // isn't one, so it leaves the game on screen with a hint: a click plays on, a second Escape pauses.
  const closed = (resume: boolean) => {
    if (resume) start();
    else resumeHint.classList.remove('hidden');
  };
  // Escape: a screen's own handler closes it first (and prevents the default). On the game with the hint it pauses; on
  // the pause screen it plays on. Escape isn't a gesture that lets a page grab the mouse again, so when the browser
  // refuses, the game stays on screen with the hint (a click plays on) rather than leaving the menu stuck.
  window.addEventListener('keydown', (e) => {
    if (e.key !== 'Escape' || e.repeat || e.defaultPrevented) return;
    if (!menu.isPaused) {
      if (resumeHint.classList.contains('hidden')) return;
      pause();
      session?.save().catch(() => {}); // pausing saves
      return;
    }
    if ((e.target as HTMLElement | null)?.closest?.('input, textarea, select') || soundLab.isOpen || panelOpen()) return;
    e.preventDefault();
    menu.el.classList.add('hidden');
    sound.unlock();
    void input.lock().then(() => {
      if (!input.locked && !document.pointerLockElement) resumeHint.classList.remove('hidden');
    });
  });
  inventory.onClose = (resume) => {
    game.close_inventory();
    closed(resume);
  };
  for (const p of closers) p.onClose = closed;
  // Browsers never let a page block Ctrl+W, so closing the tab mid-game (or just after the pointer was
  // freed by it) asks first. With the menu showing, its own links and reloads leave without asking.
  let leftGame = -Infinity;
  window.addEventListener('beforeunload', (e) => {
    if (!menu.isOpen || performance.now() - leftGame < 1000) e.preventDefault();
  });
  input.onLockChange = (locked) => {
    resumeHint.classList.add('hidden');
    if (locked || panelOpen()) menu.el.classList.add('hidden');
    else pause();
    if (!locked) {
      leftGame = performance.now();
      game.set_move(0, 0, false, false, false);
      game.pan_camera(0, 0, false);
      game.cancel_move_order();
      game.set_mining(false);
      game.cancel_line();
      game.set_using(false);
      if (!panelOpen()) session?.save().catch(() => {}); // pausing saves
    }
  };
  canvas.addEventListener('webglcontextlost', (e) => {
    e.preventDefault();
    session?.saveNow();
    pause();
    menu.say('Graphics context lost. Reload the page to continue.');
  });
  return pause;
}
