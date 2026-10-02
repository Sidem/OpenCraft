// Perspective controls and their small HUD. The engine owns modes, camera and navigation;
// this adapter routes cursor/keyboard input and keeps the mode/shortcut feedback visible.
import '../ui/view.css';
import type { Game } from '../wasm/engine';
import type { Input, Action } from '../input';
import type { Renderer } from '../render/renderer';
import type { ComfortStore } from '../comfort/settings';
import { button, h } from '../ui/dom';

const NAMES = ['First person', 'Third person', 'Strategy follow', 'Free camera'];
const ORDER_STATUS = ['', 'Walking to destination · X to stop', 'Destination reached', 'No safe route here · choose nearby dry ground', 'Route blocked · choose a new destination'];

export class ViewControls {
  readonly el = h('section', 'view-controls');
  readonly menu = h('div', 'view-menu');
  private readonly hint = h('p', 'view-hint');
  private readonly status = h('p', 'view-status');
  private readonly buttons: HTMLButtonElement[];
  private mode = -1;
  private thirdPreference: boolean | null = null;

  constructor(private readonly game: Game, private readonly input: Input, private readonly renderer: Renderer,
    private readonly canvas: HTMLCanvasElement, private readonly comfort: ComfortStore) {
    this.el.setAttribute('aria-label', 'Camera perspective');
    const row = h('div', 'view-modes');
    const select = (mode: number) => {
      game.set_view_mode(mode);
      this.sync();
    };
    this.buttons = NAMES.map((name, mode) => button('view-mode', name, () => select(mode)));
    row.append(...this.buttons);
    this.menu.append(h('p', '', 'Camera perspective · V cycles views · G frees the camera'));
    const menuRow = h('div', 'view-modes');
    const menuButtons = NAMES.map((name, mode) => button('view-mode', name, () => select(mode)));
    menuRow.append(...menuButtons);
    this.menu.append(menuRow);
    this.buttons.push(...menuButtons);
    this.el.append(row, this.hint, this.status);
    document.body.append(this.el);
    comfort.subscribe(() => {
      const s = comfort.settings;
      renderer.fovY = s.fov * Math.PI / 180;
      if (this.thirdPreference !== s.thirdPerson) {
        game.set_view_mode(s.thirdPerson ? 1 : 0);
        this.thirdPreference = s.thirdPerson;
      }
      game.set_shoulder_distance(s.thirdDistance);
      this.sync();
    });
  }

  advance(turn: number): void {
    const m = this.input.movement();
    if (this.game.view_mode() >= 2) {
      this.game.pan_camera(m.forward, m.strafe, m.sprint);
      const [x, y] = this.input.cursor;
      this.game.strategy_cursor(x, y, this.renderer.fovY, this.canvas.clientWidth / Math.max(1, this.canvas.clientHeight));
    } else {
      this.game.set_move(m.forward, m.strafe, m.jump, m.sprint, m.crouch);
      const [dx, dy] = this.input.takeLook();
      if (dx !== 0 || dy !== 0) this.game.look(dx * turn, dy * turn);
    }
    this.game.set_mining(this.input.mining);
    this.game.set_using(this.input.using);
  }

  action(a: Action): boolean {
    if (a.kind === 'view') this.game.cycle_view();
    else if (a.kind === 'free-camera') this.game.toggle_free_camera();
    else if (a.kind === 'recenter') this.game.recenter_camera();
    else if (a.kind === 'stop-order') this.game.cancel_move_order();
    else if (a.kind === 'move-order') this.game.order_move();
    else if (a.kind === 'scroll') {
      if (this.game.view_mode() >= 2) this.game.zoom_strategy(a.delta);
      else this.game.scroll_slot(a.delta);
    } else return false;
    this.sync();
    return true;
  }

  sync(): void {
    const mode = this.game.view_mode();
    if (mode !== this.mode) {
      this.mode = mode;
      if (mode < 2 && this.comfort.settings.thirdPerson !== (mode === 1)) this.comfort.set('thirdPerson', mode === 1);
      this.input.setStrategy(mode >= 2);
      document.body.classList.toggle('strategy-view', mode >= 2);
      this.buttons.forEach((b, i) => b.setAttribute('aria-pressed', String(i % 4 === mode)));
      this.hint.textContent = mode >= 2
        ? 'WASD pan · Wheel zoom · Right-click walk · Ctrl + right-click use/place · G follow/free · Home follow · V view'
        : 'V change view · G free camera';
    }
    this.status.textContent = mode >= 2
      ? (!this.game.camera_area_known() ? 'Unexplored area · only your character can reveal terrain' : ORDER_STATUS[this.game.move_order_status()] ?? '')
      : '';
    this.el.classList.toggle('playing', this.input.locked);
  }
}
