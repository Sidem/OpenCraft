// Small WebGL2 helpers: program compilation with `#define` injection and uniform location lookup.

/** Compiles and links a program. `defines` are injected right after the `#version` line. */
export function createProgram(
  gl: WebGL2RenderingContext,
  vertexSrc: string,
  fragmentSrc: string,
  defines: string[] = [],
): WebGLProgram {
  const inject = (src: string) => {
    const s = src.trim();
    const eol = s.indexOf('\n');
    return s.slice(0, eol + 1) + defines.map((d) => `#define ${d}\n`).join('') + s.slice(eol + 1);
  };
  const compile = (type: number, src: string) => {
    const sh = gl.createShader(type)!;
    gl.shaderSource(sh, inject(src));
    gl.compileShader(sh);
    if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
      throw new Error(`Shader compile failed:\n${gl.getShaderInfoLog(sh)}`);
    }
    return sh;
  };
  const prog = gl.createProgram()!;
  gl.attachShader(prog, compile(gl.VERTEX_SHADER, vertexSrc));
  gl.attachShader(prog, compile(gl.FRAGMENT_SHADER, fragmentSrc));
  gl.linkProgram(prog);
  if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
    throw new Error(`Program link failed:\n${gl.getProgramInfoLog(prog)}`);
  }
  return prog;
}

export type Uniforms<K extends string> = Record<K, WebGLUniformLocation | null>;

export function uniforms<K extends string>(gl: WebGL2RenderingContext, prog: WebGLProgram, names: readonly K[]): Uniforms<K> {
  const out = {} as Uniforms<K>;
  for (const n of names) out[n] = gl.getUniformLocation(prog, n);
  return out;
}
