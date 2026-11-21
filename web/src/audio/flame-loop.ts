// Continuous real-time pressurized flamethrower synthesizer (FIRE2): drives a seamless 4-layer Web Audio torch graph
// while the local player holds right-click with a fuelled flamethrower:
//   1. Ignition "pffft-FWOOOMPH!" one-shot burst on trigger press (valve click + gas hiss + sub-bass combustion bloom).
//   2. Deep turbulent combustion rumble (45–190 Hz) with chaotic 9.3 + 14.7 Hz flame-lick flutter.
//   3. Thick fiery mid-body roar (360 & 720 Hz resonant bands + warm soft saturation) that whooshes louder and brighter
//      when sweeping the flamethrower side-to-side through the air.
//   4. High-pressure chrome-bell venturi nozzle hiss (1.5–4.5 kHz) that snaps shut cleanly on trigger release while the
//      burning fuel roar decays naturally over ~240 ms.
// Built lazily on first ignition so sessions without a flamethrower allocate zero audio nodes.

import { mulberry32 } from './synth';
import { ignitionBurst, pinkLoop } from './synth-fire';

const LOOP_SECONDS = 4.5;

/** Warm asymmetric soft-clip curve giving fiery harmonic richness without digital harshness. */
function fireCurve(): Float32Array<ArrayBuffer> {
  const n = 256;
  const curve = new Float32Array(new ArrayBuffer(n * 4));
  for (let i = 0; i < n; i++) {
    const x = (i / (n - 1)) * 2 - 1;
    curve[i] = Math.tanh(x * 2.4 + 0.18 * x * x);
  }
  return curve;
}

export class FlameLoop {
  private ctx: AudioContext | null = null;
  private out: GainNode | null = null;
  private ready = false;
  private burning = false;
  private lastYaw = 0;
  private lastPitch = 0;
  private hasLook = false;
  private sweep = 0;

  private ignBuf: AudioBuffer | null = null;
  private rumbleGain: GainNode | null = null;
  private midGain: GainNode | null = null;
  private hissGain: GainNode | null = null;
  private midHiFilter: BiquadFilterNode | null = null;
  private midLoFilter: BiquadFilterNode | null = null;
  private rumbleFilter: BiquadFilterNode | null = null;
  private hissFilter: BiquadFilterNode | null = null;
  private flutterGain: GainNode | null = null;

  start(ctx: AudioContext, effects: GainNode): void {
    this.ctx = ctx;
    this.out = effects;
  }

  /** Called every frame from `SoundSystem.poll`: tracks burning state, aim-sweep speed, and flame action dials. */
  poll(active: boolean, yaw: number, pitch: number, dt: number, volumeGain: number, pitchScale: number): void {
    const ctx = this.ctx;
    if (!ctx || !this.out || ctx.state !== 'running') {
      this.burning = false;
      return;
    }
    const now = ctx.currentTime;

    // Track angular aim speed for realistic rushing air-flame WHOOSH when sweeping side-to-side.
    if (this.hasLook && dt > 1e-4) {
      let dy = yaw - this.lastYaw;
      if (dy > Math.PI) dy -= Math.PI * 2;
      if (dy < -Math.PI) dy += Math.PI * 2;
      const dp = pitch - this.lastPitch;
      const angVel = Math.hypot(dy, dp) / dt;
      const target = Math.min(1, Math.max(0, (angVel - 0.35) / 3.2));
      this.sweep += (target - this.sweep) * Math.min(1, dt * 10);
    }
    this.lastYaw = yaw;
    this.lastPitch = pitch;
    this.hasLook = true;

    if (!active) {
      if (this.burning && this.ready) {
        // Valve shuts crisply on the chrome bell hiss (35 ms) while the burning orange plume trails out (~180 ms).
        this.hissGain!.gain.setTargetAtTime(0, now, 0.022);
        this.midGain!.gain.setTargetAtTime(0, now, 0.085);
        this.rumbleGain!.gain.setTargetAtTime(0, now, 0.11);
      }
      this.burning = false;
      return;
    }

    if (!this.ready) this.build(ctx, this.out);

    // Rising edge: trigger the crisp valve-open + sub-bass combustion ignition "pffft-FWOOOMPH!".
    if (!this.burning && this.ignBuf && volumeGain > 0) {
      const src = ctx.createBufferSource();
      src.buffer = this.ignBuf;
      src.playbackRate.value = pitchScale * (0.96 + Math.random() * 0.08);
      const g = ctx.createGain();
      g.gain.value = 0.62 * volumeGain;
      src.connect(g).connect(this.out);
      src.onended = () => {
        src.disconnect();
        g.disconnect();
      };
      src.start(now);
    }
    this.burning = true;

    // Continuous 3-band roaring torch mix, modulated dynamically by sweep speed and the flame action dials.
    const sw = this.sweep;
    const base = 0.56 * volumeGain;
    this.rumbleFilter!.frequency.setTargetAtTime((175 + sw * 65) * pitchScale, now, 0.04);
    this.midLoFilter!.frequency.setTargetAtTime((350 + sw * 170) * pitchScale, now, 0.04);
    this.midHiFilter!.frequency.setTargetAtTime((720 + sw * 580) * pitchScale, now, 0.04);
    this.hissFilter!.frequency.setTargetAtTime((2650 + sw * 550) * pitchScale, now, 0.04);

    // Subtle ramp up during initial ignition, plus big fiery swell during fast sweeps.
    this.rumbleGain!.gain.setTargetAtTime(base * (1.15 + sw * 0.30), now, 0.055);
    this.midGain!.gain.setTargetAtTime(base * (0.90 + sw * 0.48), now, 0.045);
    this.hissGain!.gain.setTargetAtTime(base * (0.28 + sw * 0.08), now, 0.025);
    this.flutterGain!.gain.setTargetAtTime(0.24 + sw * 0.14, now, 0.05);
  }

  private build(ctx: AudioContext, dest: GainNode): void {
    const sr = ctx.sampleRate;
    const rand = mulberry32(0xf1a3e50);
    const loopData = pinkLoop(sr, LOOP_SECONDS, rand);
    const loopBuf = ctx.createBuffer(1, loopData.length, sr);
    loopBuf.getChannelData(0).set(loopData);

    const ignData = ignitionBurst(sr, mulberry32(0xf1a316), 1);
    this.ignBuf = ctx.createBuffer(1, ignData.length, sr);
    this.ignBuf.getChannelData(0).set(ignData);

    // Master noise loop feeding all 3 continuous torch bands simultaneously (zero phase beat, zero repeat).
    const src = ctx.createBufferSource();
    src.buffer = loopBuf;
    src.loop = true;

    // Chaotic turbulent flame-lick flutter (two non-harmonic LFOs summing into rumble + mid amplitude).
    const lfo1 = ctx.createOscillator();
    lfo1.type = 'sine';
    lfo1.frequency.value = 9.3;
    const lfo2 = ctx.createOscillator();
    lfo2.type = 'triangle';
    lfo2.frequency.value = 14.7;
    const flutterDepth = ctx.createGain();
    flutterDepth.gain.value = 0.24;
    const flutterMod = ctx.createGain();
    flutterMod.gain.value = 1.0;
    lfo1.connect(flutterDepth);
    lfo2.connect(flutterDepth);
    flutterDepth.connect(flutterMod.gain);
    this.flutterGain = flutterDepth;

    // Band 1: Deep turbulent combustion rumble (45–185 Hz).
    const rumbleLp = ctx.createBiquadFilter();
    rumbleLp.type = 'lowpass';
    rumbleLp.frequency.value = 175;
    rumbleLp.Q.value = 1.6;
    const rumbleGain = ctx.createGain();
    rumbleGain.gain.value = 0;

    // Band 2: Thick fiery mid-body roar (dual resonant bands + warm saturation).
    const midLo = ctx.createBiquadFilter();
    midLo.type = 'bandpass';
    midLo.frequency.value = 350;
    midLo.Q.value = 1.25;
    const midHi = ctx.createBiquadFilter();
    midHi.type = 'bandpass';
    midHi.frequency.value = 720;
    midHi.Q.value = 1.45;
    const shaper = ctx.createWaveShaper();
    shaper.curve = fireCurve();
    const midGain = ctx.createGain();
    midGain.gain.value = 0;

    // Band 3: High-pressure chrome-bell venturi nozzle hiss.
    const hissHp = ctx.createBiquadFilter();
    hissHp.type = 'highpass';
    hissHp.frequency.value = 1400;
    const hissBp = ctx.createBiquadFilter();
    hissBp.type = 'bandpass';
    hissBp.frequency.value = 2650;
    hissBp.Q.value = 0.85;
    const hissGain = ctx.createGain();
    hissGain.gain.value = 0;

    src.connect(rumbleLp).connect(flutterMod).connect(rumbleGain).connect(dest);
    src.connect(midLo).connect(shaper);
    src.connect(midHi).connect(shaper);
    shaper.connect(midGain).connect(dest);
    src.connect(hissHp).connect(hissBp).connect(hissGain).connect(dest);

    src.start();
    lfo1.start();
    lfo2.start();

    this.rumbleFilter = rumbleLp;
    this.midLoFilter = midLo;
    this.midHiFilter = midHi;
    this.hissFilter = hissBp;
    this.rumbleGain = rumbleGain;
    this.midGain = midGain;
    this.hissGain = hissGain;
    this.ready = true;
  }
}
