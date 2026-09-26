// Preview-only 3×3 world-material samples rendered through the production terrain fragment shader.
// This shows the continuous world variation that a raw repeated atlas tile cannot represent, and a
// layer's alternate looks (one per block, as the mesher picks them) when it has any.
import { litFrag } from '../src/render/shaders.ts';

// `sampleLooks`: per sample, its layer followed by its alternates; results are keyed by the layer.
export function worldMaterialSamples(pixels, size, layers, sampleLooks) {
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = 144;
  const gl = canvas.getContext('webgl2', { antialias: false, preserveDrawingBuffer: true });
  if (!gl) throw new Error('WebGL2 is required for the material review');
  const vertex = `#version 300 es
precision highp float;
layout(location=0) in vec2 a_pos;
uniform float u_layer;
uniform vec2 u_cell;
out vec3 v_uvl;
out float v_light;
out vec3 v_rel;
out vec2 v_ground;
void main() {
  vec2 world = vec2(20.0, 8.0) + u_cell + (a_pos * 0.5 + 0.5);
  v_uvl = vec3(world, u_layer);
  v_ground = world;
  v_light = 1.0;
  v_rel = vec3(0.0);
  gl_Position = vec4(a_pos, 0.0, 1.0);
}`;
  const fragment = litFrag.replace('#version 300 es', '#version 300 es\n#define TERRAIN\n#define CUTOUT');
  const compile = (kind, source) => {
    const shader = gl.createShader(kind);
    gl.shaderSource(shader, source); gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) throw new Error(gl.getShaderInfoLog(shader));
    return shader;
  };
  const program = gl.createProgram();
  gl.attachShader(program, compile(gl.VERTEX_SHADER, vertex));
  gl.attachShader(program, compile(gl.FRAGMENT_SHADER, fragment));
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) throw new Error(gl.getProgramInfoLog(program));
  gl.useProgram(program);
  gl.uniform3f(gl.getUniformLocation(program, 'u_fogColor'), 0, 0, 0);
  gl.uniform2f(gl.getUniformLocation(program, 'u_fog'), 100, 200);
  gl.uniform1i(gl.getUniformLocation(program, 'u_tex'), 0);
  const buffer = gl.createBuffer(); gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1, 1,-1, -1,1, 1,1]), gl.STATIC_DRAW);
  gl.enableVertexAttribArray(0); gl.vertexAttribPointer(0, 2, gl.FLOAT, false, 0, 0);
  const texture = gl.createTexture(); gl.activeTexture(gl.TEXTURE0); gl.bindTexture(gl.TEXTURE_2D_ARRAY, texture);
  gl.texImage3D(gl.TEXTURE_2D_ARRAY, 0, gl.RGBA8, size, size, layers, 0, gl.RGBA, gl.UNSIGNED_BYTE, pixels);
  gl.generateMipmap(gl.TEXTURE_2D_ARRAY);
  gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MIN_FILTER, gl.NEAREST_MIPMAP_LINEAR);
  gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
  gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_S, gl.REPEAT);
  gl.texParameteri(gl.TEXTURE_2D_ARRAY, gl.TEXTURE_WRAP_T, gl.REPEAT);
  const result = new Map(), cell = canvas.width / 3;
  for (const looks of sampleLooks) {
    gl.clearColor(0.53, 0.58, 0.59, 1); gl.clear(gl.COLOR_BUFFER_BIT);
    for (let i = 0; i < 9; i++) {
      const [cx, cy] = [i % 3, Math.floor(i / 3)];
      gl.viewport(cx * cell, cy * cell, cell, cell);
      gl.uniform2f(gl.getUniformLocation(program, 'u_cell'), cx, cy);
      gl.uniform1f(gl.getUniformLocation(program, 'u_layer'), looks[(i * 7 + 3) % 9 % looks.length]);
      gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
    }
    const raw = new Uint8Array(canvas.width * canvas.height * 4);
    gl.readPixels(0, 0, canvas.width, canvas.height, gl.RGBA, gl.UNSIGNED_BYTE, raw);
    const flipped = new Uint8ClampedArray(raw.length), stride = canvas.width * 4;
    for (let y = 0; y < canvas.height; y++) {
      flipped.set(raw.subarray((canvas.height - 1 - y) * stride, (canvas.height - y) * stride), y * stride);
    }
    result.set(looks[0], new ImageData(flipped, canvas.width, canvas.height));
  }
  gl.deleteTexture(texture); gl.deleteBuffer(buffer); gl.deleteProgram(program);
  return result;
}
