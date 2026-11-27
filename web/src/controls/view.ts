// First-person camera and hand adapter: routes movement, mouse look, both hands (G works the left: engine
// left_hand.rs) and Shift+Wheel shape/paint cycling to the engine, and syncs field-of-view and first-person hand visibility from ComfortStore.

import type { ComfortStore } from '../comfort/settings';
import type { Action, Input } from '../input';
import type { Renderer } from '../render/renderer';
import type { Game } from '../wasm/engine';

export class ViewControls {
  constructor(
    private readonly game: Game,
    private readonly input: Input,
    private readonly renderer: Renderer,
    comfort: ComfortStore,
  ) {
    game.set_view_mode(0);
    comfort.subscribe(() => {
      const s = comfort.settings;
      this.renderer.fovY = (s.fov * Math.PI) / 180;
      this.game.set_view_mode(0);
      this.game.set_show_hand(s.showHand);
    });
  }

  advance(turn: number): void {
    const m = this.input.movement();
    this.game.set_move(m.forward, m.strafe, m.jump, m.sprint, m.crouch);
    const [dx, dy] = this.input.takeLook();
    if (dx !== 0 || dy !== 0) this.game.look(dx * turn, dy * turn);
    this.game.set_mining(this.input.mining);
    this.game.set_using(this.input.using);
    this.game.set_left_using(this.input.leftUsing);
  }

  action(a: Action): boolean {
    if (a.kind !== 'scroll') return false;
    if (!a.shift || !this.game.cycle_shape(a.delta)) this.game.scroll_slot(a.delta);
    return true;
  }
}
