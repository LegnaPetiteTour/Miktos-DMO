/**
 * PhysarumSim — WebGL2 Physarum polycephalum particle simulation.
 *
 * Implements the Jones (2010) multi-agent trail-following model:
 *   - Agents sense trail + chemoattractant at 3 probe points (F, FL, FR)
 *   - Rotate toward highest signal by rotation angle RA
 *   - Move forward by stepSize pixels, wrap at boundaries
 *   - Deposit trail at new position
 *   - Trail diffuses (3×3 Gaussian blur) and decays each frame
 *
 * Architecture:
 *   - Agent state : 2D RGBA32F texture (agentW × agentH), ping-pong
 *   - Trail map   : RGBA32F texture (simWidth × simHeight), ping-pong
 *   - Chemo map   : RGBA32F texture (simWidth × simHeight), uploaded once per scan
 *
 * Shader passes per frame:
 *   1. Agent update  — agentW×agentH viewport, overwrite
 *   2. Diffuse+decay — simWidth×simHeight viewport, overwrite
 *   3. Deposit       — simWidth×simHeight viewport, additive blend, POINTS
 *   4. Render        — simWidth×simHeight viewport, outputs to canvas
 *
 * Default Jones (2010) parameters:
 *   SA = 45°  SO = 9px  RA = 45°  stepSize = 1.0
 */

// ─────────────────────────────── Types ────────────────────────────────────

export interface SimParams {
  /** Number of slime agents. Default: 50 000 */
  agentCount: number;
  /** Sensor angle offset in radians. Default: Math.PI / 4 (45°) */
  sa: number;
  /** Sensor offset distance in pixels. Default: 9 */
  so: number;
  /** Rotation angle in radians. Default: Math.PI / 4 (45°) */
  ra: number;
  /** Step size in pixels per frame. Default: 1.0 */
  stepSize: number;
  /** Trail deposit amount per agent per frame. Default: 0.05 */
  deposit: number;
  /** Trail decay multiplier per frame (< 1). Default: 0.95 */
  decayRate: number;
  /** Chemoattractant weight relative to trail during sensing. Default: 2.0 */
  chemoWeight: number;
}

const DEFAULT_PARAMS: SimParams = {
  agentCount: 50_000,
  sa: Math.PI / 4,
  so: 9,
  ra: Math.PI / 4,
  stepSize: 1.0,
  deposit: 0.05,
  decayRate: 0.95,
  chemoWeight: 2.0,
};

// ─────────────────────────────── Shaders ──────────────────────────────────

/** Full-screen quad vertex shader (attribute-less, uses gl_VertexID 0-5). */
const QUAD_VERT = `#version 300 es
void main() {
  float x = float((gl_VertexID & 1) << 1) - 1.0;
  float y = float((gl_VertexID & 2)) - 1.0;
  // Vertices: (−1,−1), (1,−1), (−1,1), (1,−1), (1,1), (−1,1)
  vec2 pos[6];
  pos[0] = vec2(-1.0, -1.0);
  pos[1] = vec2( 1.0, -1.0);
  pos[2] = vec2(-1.0,  1.0);
  pos[3] = vec2(-1.0,  1.0);
  pos[4] = vec2( 1.0, -1.0);
  pos[5] = vec2( 1.0,  1.0);
  gl_Position = vec4(pos[gl_VertexID], 0.0, 1.0);
}`;

/**
 * Deposit vertex shader.
 * Each vertex = one agent. Uses gl_VertexID to index the 2D agent texture,
 * then places a point at the agent's canvas position.
 */
const DEPOSIT_VERT = `#version 300 es
uniform sampler2D u_agents;
uniform vec2 u_size;          // canvas (width, height)
uniform ivec2 u_agent_dims;   // (agentW, agentH)
uniform int u_agent_count;

void main() {
  if (gl_VertexID >= u_agent_count) {
    gl_Position = vec4(-99.0, -99.0, 0.0, 1.0); // off-screen
    gl_PointSize = 0.0;
    return;
  }
  int ax = gl_VertexID % u_agent_dims.x;
  int ay = gl_VertexID / u_agent_dims.x;
  vec4 agent = texelFetch(u_agents, ivec2(ax, ay), 0);
  // Convert pixel position to NDC
  vec2 clip = (agent.xy / u_size) * 2.0 - 1.0;
  gl_Position = vec4(clip, 0.0, 1.0);
  gl_PointSize = 2.0;
}`;

/**
 * Agent update fragment shader.
 * Renders to the agent texture (one fragment = one agent).
 * Senses trail+chemo at three probes, rotates, moves, wraps.
 */
const AGENT_FRAG = `#version 300 es
precision highp float;

uniform sampler2D u_agents;
uniform sampler2D u_trail;
uniform sampler2D u_chemo;
uniform vec2 u_size;
uniform ivec2 u_agent_dims;
uniform float u_sa;
uniform float u_so;
uniform float u_ra;
uniform float u_step;
uniform float u_chemo_weight;
uniform float u_time;

out vec4 fragColor;

float sense(vec2 pos, float angle) {
  vec2 probe = pos + vec2(cos(angle), sin(angle)) * u_so;
  vec2 uv = clamp(probe / u_size, 0.0, 1.0);
  return texture(u_trail, uv).r + texture(u_chemo, uv).r * u_chemo_weight;
}

float rng(vec2 p) {
  return fract(sin(dot(p + u_time, vec2(127.1, 311.7))) * 43758.5453);
}

void main() {
  ivec2 coord = ivec2(gl_FragCoord.xy);
  // Skip extra padding texels beyond agentCount
  if (coord.x >= u_agent_dims.x || coord.y >= u_agent_dims.y) {
    fragColor = vec4(0.0);
    return;
  }
  vec4 agent = texelFetch(u_agents, coord, 0);
  vec2 pos = agent.xy;
  float h = agent.z;

  float fwd = sense(pos, h);
  float fl  = sense(pos, h + u_sa);
  float fr  = sense(pos, h - u_sa);

  if (fwd >= fl && fwd >= fr) {
    // Go straight
  } else if (fl > fr) {
    h += u_ra;
  } else if (fr > fl) {
    h -= u_ra;
  } else {
    // Tie: random turn
    h += (rng(pos) > 0.5 ? u_ra : -u_ra);
  }

  vec2 newPos = mod(pos + vec2(cos(h), sin(h)) * u_step + u_size, u_size);
  fragColor = vec4(newPos, h, agent.w);
}`;

/** Deposit fragment shader — writes a constant deposit value at each point. */
const DEPOSIT_FRAG = `#version 300 es
precision highp float;
uniform float u_deposit;
out vec4 fragColor;
void main() {
  fragColor = vec4(u_deposit, 0.0, 0.0, 0.0);
}`;

/** Diffuse + decay fragment shader — 3×3 box blur of trail map + exponential decay. */
const DIFFUSE_FRAG = `#version 300 es
precision highp float;
uniform sampler2D u_trail;
uniform vec2 u_resolution;
uniform float u_decay;
out vec4 fragColor;
void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  vec2 px = 1.0 / u_resolution;
  float sum = 0.0;
  for (int dy = -1; dy <= 1; dy++) {
    for (int dx = -1; dx <= 1; dx++) {
      sum += texture(u_trail, uv + vec2(float(dx), float(dy)) * px).r;
    }
  }
  float blurred = sum / 9.0;
  fragColor = vec4(clamp(blurred * u_decay, 0.0, 1.0), 0.0, 0.0, 0.0);
}`;

/**
 * Render fragment shader.
 * Maps trail concentration to a cyan→white glow; chemo adds a warm hint.
 * Outputs premultiplied RGBA so the canvas alpha-composites over the treemap.
 */
const RENDER_FRAG = `#version 300 es
precision highp float;
uniform sampler2D u_trail;
uniform sampler2D u_chemo;
uniform vec2 u_resolution;
out vec4 fragColor;
void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;

  float trail = texture(u_trail, uv).r;
  float chemo = texture(u_chemo, uv).r;

  // Boost trails so even sparse coverage is visible
  float tv = min(trail * 20.0, 1.0);
  float glow = pow(tv, 0.45);

  // Color: dim cyan → bright white as trail intensifies
  vec3 col = mix(vec3(0.05, 0.55, 1.0), vec3(1.0, 1.0, 1.0), pow(tv, 2.5));
  // Warm chemoattractant substrate hint (does not obscure terrain below)
  col += vec3(0.9, 0.25, 0.05) * chemo * 0.14;

  float alpha = glow * 0.92;
  // Premultiplied alpha for correct canvas composite
  fragColor = vec4(col * alpha, alpha);
}`;

// ─────────────────────────────── Class ────────────────────────────────────

export class PhysarumSim {
  private readonly gl: WebGL2RenderingContext;
  private readonly width: number;
  private readonly height: number;
  private readonly params: SimParams;
  private readonly agentCount: number;
  private readonly agentW: number;  // agent texture width (≈√agentCount)
  private readonly agentH: number;  // agent texture height

  private progAgent!: WebGLProgram;
  private progDeposit!: WebGLProgram;
  private progDiffuse!: WebGLProgram;
  private progRender!: WebGLProgram;

  private agentTexA!: WebGLTexture;
  private agentTexB!: WebGLTexture;
  private trailTexA!: WebGLTexture;
  private trailTexB!: WebGLTexture;
  private chemoTex!: WebGLTexture;

  private fbAgentA!: WebGLFramebuffer;
  private fbAgentB!: WebGLFramebuffer;
  private fbTrailA!: WebGLFramebuffer;
  private fbTrailB!: WebGLFramebuffer;

  private vao!: WebGLVertexArrayObject;

  private agentPing = true;  // true → read A, write B
  private trailPing = true;  // true → read A, write B
  private time = 0;

  constructor(canvas: HTMLCanvasElement, params?: Partial<SimParams>) {
    const gl = canvas.getContext("webgl2");
    if (!gl) throw new Error("WebGL2 is not available in this browser.");

    const ext = gl.getExtension("EXT_color_buffer_float");
    if (!ext) throw new Error("EXT_color_buffer_float is required for the simulation.");

    this.gl = gl;
    this.width = canvas.width;
    this.height = canvas.height;
    this.params = { ...DEFAULT_PARAMS, ...params };
    this.agentCount = this.params.agentCount;

    // 2-D layout for agent texture (avoids max-texture-width limit)
    this.agentW = Math.ceil(Math.sqrt(this.agentCount));
    this.agentH = Math.ceil(this.agentCount / this.agentW);

    this.initGL();
    this.initAgents();
  }

  // ── Public API ────────────────────────────────────────────────────────

  /** Advance simulation by one frame and render to the canvas. */
  step(): void {
    const { gl, params, agentCount, agentW, agentH, width, height } = this;
    this.time += 0.016;

    gl.bindVertexArray(this.vao);

    // Current ping-pong pointers (before any flip this frame)
    const agentR   = this.agentPing ? this.agentTexA : this.agentTexB;
    const agentWfb = this.agentPing ? this.fbAgentB   : this.fbAgentA;
    const trailR   = this.trailPing ? this.trailTexA  : this.trailTexB;
    const trailWfb = this.trailPing ? this.fbTrailB   : this.fbTrailA;

    // ── Pass 1: Agent sensing + movement → agentW×agentH ────────────────
    gl.bindFramebuffer(gl.FRAMEBUFFER, agentWfb);
    gl.viewport(0, 0, agentW, agentH);
    gl.disable(gl.BLEND);
    gl.useProgram(this.progAgent);
    this.bindTex(gl, 0, agentR);
    this.bindTex(gl, 1, trailR);
    this.bindTex(gl, 2, this.chemoTex);
    this.setUni1i(this.progAgent, "u_agents",      0);
    this.setUni1i(this.progAgent, "u_trail",       1);
    this.setUni1i(this.progAgent, "u_chemo",       2);
    this.setUni2f(this.progAgent, "u_size",        width, height);
    this.setUni2i(this.progAgent, "u_agent_dims",  agentW, agentH);
    this.setUni1f(this.progAgent, "u_sa",          params.sa);
    this.setUni1f(this.progAgent, "u_so",          params.so);
    this.setUni1f(this.progAgent, "u_ra",          params.ra);
    this.setUni1f(this.progAgent, "u_step",        params.stepSize);
    this.setUni1f(this.progAgent, "u_chemo_weight",params.chemoWeight);
    this.setUni1f(this.progAgent, "u_time",        this.time);
    gl.drawArrays(gl.TRIANGLES, 0, 6);

    // Flip agent ping-pong; from here on, agentR_new refers to freshly written tex
    this.agentPing = !this.agentPing;
    const agentNew = this.agentPing ? this.agentTexA : this.agentTexB;

    // ── Pass 2: Trail diffuse + decay → width×height ─────────────────────
    gl.bindFramebuffer(gl.FRAMEBUFFER, trailWfb);
    gl.viewport(0, 0, width, height);
    gl.disable(gl.BLEND);
    gl.useProgram(this.progDiffuse);
    this.bindTex(gl, 0, trailR);
    this.setUni1i(this.progDiffuse,  "u_trail",      0);
    this.setUni2f(this.progDiffuse,  "u_resolution", width, height);
    this.setUni1f(this.progDiffuse,  "u_decay",      params.decayRate);
    gl.drawArrays(gl.TRIANGLES, 0, 6);

    // ── Pass 3: Deposit (additive blend on top of diffused trail) ────────
    gl.enable(gl.BLEND);
    gl.blendEquation(gl.FUNC_ADD);
    gl.blendFunc(gl.ONE, gl.ONE);
    gl.useProgram(this.progDeposit);
    this.bindTex(gl, 0, agentNew);
    this.setUni1i(this.progDeposit, "u_agents",     0);
    this.setUni2f(this.progDeposit, "u_size",       width, height);
    this.setUni2i(this.progDeposit, "u_agent_dims", agentW, agentH);
    this.setUni1i(this.progDeposit, "u_agent_count",agentCount);
    this.setUni1f(this.progDeposit, "u_deposit",    params.deposit);
    gl.drawArrays(gl.POINTS, 0, agentCount);
    gl.disable(gl.BLEND);

    // Flip trail ping-pong; the written texture becomes the new read source
    this.trailPing = !this.trailPing;
    const trailNew = this.trailPing ? this.trailTexA : this.trailTexB;

    // ── Pass 4: Render trail to canvas ───────────────────────────────────
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    gl.viewport(0, 0, width, height);
    gl.clearColor(0, 0, 0, 0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.useProgram(this.progRender);
    this.bindTex(gl, 0, trailNew);
    this.bindTex(gl, 1, this.chemoTex);
    this.setUni1i(this.progRender, "u_trail",      0);
    this.setUni1i(this.progRender, "u_chemo",      1);
    this.setUni2f(this.progRender, "u_resolution", width, height);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  }

  /**
   * Upload a new chemoattractant map from buildChemoattractantTexture().
   * Can be called at any time (e.g. when terrain updates after a re-scan).
   */
  updateChemoMap(data: Float32Array): void {
    const { gl, width, height } = this;
    // data is width*height floats; expand to RGBA32F (only R is used)
    const rgba = new Float32Array(width * height * 4);
    for (let i = 0; i < width * height; i++) rgba[i * 4] = data[i];

    gl.bindTexture(gl.TEXTURE_2D, this.chemoTex);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, width, height, gl.RGBA, gl.FLOAT, rgba);
    gl.bindTexture(gl.TEXTURE_2D, null);
  }

  /** Release all WebGL resources. Must be called when the simulation stops. */
  dispose(): void {
    const { gl } = this;
    [this.progAgent, this.progDeposit, this.progDiffuse, this.progRender]
      .forEach(p => gl.deleteProgram(p));
    [this.agentTexA, this.agentTexB, this.trailTexA, this.trailTexB, this.chemoTex]
      .forEach(t => gl.deleteTexture(t));
    [this.fbAgentA, this.fbAgentB, this.fbTrailA, this.fbTrailB]
      .forEach(f => gl.deleteFramebuffer(f));
    gl.deleteVertexArray(this.vao);
  }

  // ── Private helpers ───────────────────────────────────────────────────

  private initGL(): void {
    const { gl, width, height, agentW, agentH } = this;

    // Compile programs
    this.progAgent   = this.compileProgram(QUAD_VERT,    AGENT_FRAG);
    this.progDeposit = this.compileProgram(DEPOSIT_VERT, DEPOSIT_FRAG);
    this.progDiffuse = this.compileProgram(QUAD_VERT,    DIFFUSE_FRAG);
    this.progRender  = this.compileProgram(QUAD_VERT,    RENDER_FRAG);

    // Null VAO for attribute-less draws
    this.vao = gl.createVertexArray()!;

    // Agent textures (RGBA32F, agentW × agentH)
    this.agentTexA = this.makeFloatTex(agentW, agentH);
    this.agentTexB = this.makeFloatTex(agentW, agentH);
    this.fbAgentA  = this.makeFB(this.agentTexA);
    this.fbAgentB  = this.makeFB(this.agentTexB);

    // Trail textures (RGBA32F, width × height)
    this.trailTexA = this.makeFloatTex(width, height);
    this.trailTexB = this.makeFloatTex(width, height);
    this.fbTrailA  = this.makeFB(this.trailTexA);
    this.fbTrailB  = this.makeFB(this.trailTexB);

    // Chemo texture (RGBA32F, width × height) — populated via updateChemoMap()
    this.chemoTex = this.makeFloatTex(width, height);
  }

  private initAgents(): void {
    const { gl, agentCount, agentW, agentH, width, height } = this;
    const totalSlots = agentW * agentH;
    const data = new Float32Array(totalSlots * 4);
    for (let i = 0; i < agentCount; i++) {
      data[i * 4 + 0] = Math.random() * width;
      data[i * 4 + 1] = Math.random() * height;
      data[i * 4 + 2] = Math.random() * Math.PI * 2;
      data[i * 4 + 3] = 1.0;
    }
    gl.bindTexture(gl.TEXTURE_2D, this.agentTexA);
    gl.texSubImage2D(gl.TEXTURE_2D, 0, 0, 0, agentW, agentH, gl.RGBA, gl.FLOAT, data);
    gl.bindTexture(gl.TEXTURE_2D, null);
  }

  private makeFloatTex(w: number, h: number): WebGLTexture {
    const { gl } = this;
    const tex = gl.createTexture()!;
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA32F, w, h, 0, gl.RGBA, gl.FLOAT, null);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.bindTexture(gl.TEXTURE_2D, null);
    return tex;
  }

  private makeFB(tex: WebGLTexture): WebGLFramebuffer {
    const { gl } = this;
    const fb = gl.createFramebuffer()!;
    gl.bindFramebuffer(gl.FRAMEBUFFER, fb);
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, tex, 0);
    const status = gl.checkFramebufferStatus(gl.FRAMEBUFFER);
    if (status !== gl.FRAMEBUFFER_COMPLETE) {
      throw new Error(`Framebuffer incomplete: 0x${status.toString(16)}`);
    }
    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
    return fb;
  }

  private compileProgram(vertSrc: string, fragSrc: string): WebGLProgram {
    const { gl } = this;
    const vert = this.compileShader(gl.VERTEX_SHADER,   vertSrc);
    const frag = this.compileShader(gl.FRAGMENT_SHADER, fragSrc);
    const prog = gl.createProgram()!;
    gl.attachShader(prog, vert);
    gl.attachShader(prog, frag);
    gl.linkProgram(prog);
    gl.deleteShader(vert);
    gl.deleteShader(frag);
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
      throw new Error(`Program link failed: ${gl.getProgramInfoLog(prog)}`);
    }
    return prog;
  }

  private compileShader(type: number, src: string): WebGLShader {
    const { gl } = this;
    const shader = gl.createShader(type)!;
    gl.shaderSource(shader, src);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      throw new Error(`Shader compile failed: ${gl.getShaderInfoLog(shader)}\n\n${src}`);
    }
    return shader;
  }

  private bindTex(gl: WebGL2RenderingContext, unit: number, tex: WebGLTexture): void {
    gl.activeTexture(gl.TEXTURE0 + unit);
    gl.bindTexture(gl.TEXTURE_2D, tex);
  }

  private ul(prog: WebGLProgram, name: string): WebGLUniformLocation | null {
    return this.gl.getUniformLocation(prog, name);
  }

  private setUni1i(prog: WebGLProgram, name: string, v: number): void {
    this.gl.uniform1i(this.ul(prog, name), v);
  }
  private setUni1f(prog: WebGLProgram, name: string, v: number): void {
    this.gl.uniform1f(this.ul(prog, name), v);
  }
  private setUni2f(prog: WebGLProgram, name: string, x: number, y: number): void {
    this.gl.uniform2f(this.ul(prog, name), x, y);
  }
  private setUni2i(prog: WebGLProgram, name: string, x: number, y: number): void {
    this.gl.uniform2i(this.ul(prog, name), x, y);
  }
}
