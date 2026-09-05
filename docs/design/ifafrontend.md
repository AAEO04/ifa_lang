# Ifafrontend — Declarative GPU-Accelerated UI Framework

**Status:** Design phase (not implemented)

A write-once declarative UI framework targeting terminal (ratatui), native desktop (wgpu + winit), and browser (wgpu + wasm) from a single Ifá-Lang codebase.

---

## Architecture

```
ifafrontend.ifa (declarative Ifá-Lang API)
    │
    ▼
Ose dispatch (crates/ifa-std/src/odu/ose.rs)
    │
    ├── #[cfg(feature = "tui")] ── ratatui renderer (existing)
    │                                  │
    │                                  └── Termion/Crossterm backend
    │
    └── #[cfg(feature = "webgpu-ui")] ── stacks/frontend.rs engine (new)
                                           │
                                           ├── SceneGraph (DOM-like tree)
                                           ├── LayoutEngine (Taffy)
                                           ├── QuadBatcher (wgpu instanced)
                                           ├── TextEngine (fontdue / browser FFI)
                                           ├── AssetCache (textures, fonts)
                                           ├── InputMapper (winit / browser events)
                                           └── Renderer (wgpu surface → swapchain)
```

### Three Output Modes, One Layout Engine

| Mode | Backend | Surface | Input |
|------|---------|---------|-------|
| Terminal | ratatui + crossterm | stdio alternate screen | keyboard, mouse |
| Desktop | wgpu + winit | native window (glfw/x11/wayland) | winit events |
| Browser | wgpu + wasm-bindgen | `<canvas>` WebGPU | DOM events → JS FFI |

All three share Taffy layout, diffing only in the render pass. The `ifafrontend.ifa` API is identical for all three.

### Unique Differentiators

1. **Opon epoch bulk-free per frame** — Scene graph nodes and render data are allocated in an Opon slot. When the frame finishes rendering, the epoch truncates. Zero GC pressure during UI updates.
2. **Ajose reactive signals** — Fine-grained push-based reactivity. Signals created in `ifafrontend.ifa` propagate changes directly to the scene graph, skipping the virtual DOM diff entirely.
3. **JS FFI bridge** — Access the entire JavaScript ecosystem (Three.js, Chart.js, D3, etc.) from Ifá-Lang without reimplementing anything. JSON-serialized calls cross the bridge.
4. **Write once → three outputs** — The same `.ifa` file runs in the terminal, as a native desktop app, or in the browser, with platform-appropriate rendering.

---

## Module Breakdown: `stacks/frontend.rs`

### 1. SceneGraph (`~400 lines`)

A DOM-like tree of typed nodes, allocated in an Opon epoch:

```rust
pub enum SceneNode {
    Div(ContainerNode),
    Text(TextNode),
    Image(ImageNode),
    Canvas(CanvasNode),       // JS FFI offscreen canvas
    ThreeViewport(ThreeNode), // Three.js integration point
    Custom(CustomNode),       // wgpu escape hatch
}

pub struct ContainerNode {
    pub id: Option<String>,
    pub style: StyleBox,
    pub children: Vec<SceneNode>,
    pub ajose_subscriptions: Vec<SubscriptionId>,
}
```

- No V-DOM diffing — mutations flow directly from Ajose signals to node properties
- Nodes live in an Opon arena; the entire tree is bulk-freed when the epoch ends
- `SceneNode::ThreeViewport` is a placeholder rectangle where Three.js renders

### 2. LayoutEngine (`~150 lines`)

Thin wrapper around Taffy (`crates.io/taffy`):

```rust
pub struct LayoutEngine {
    taffy: taffy::TaffyTree,
    style_cache: HashMap<NodeId, taffy::Style>,
}
```

- Taffy computes flexbox layout from the same `StyleBox` used by ratatui
- Shared between terminal and GPU renderers
- Called once per frame after scene graph mutations settle
- Output: absolute positioned rectangles for each node

### 3. QuadBatcher (`~350 lines`)

Instanced quad rendering with batching:

```rust
pub struct QuadBatcher {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
}

struct Instance {
    position: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    texture_id: Option<u32>,
    border_radius: [f32; 4],
    z_index: i32,
}
```

- Rects, borders, backgrounds, rounded corners — all quads
- Sorted by z-index, then by texture to minimize draw calls
- One instanced draw call per texture group (typically 1–5 calls per frame)
- WGSL vertex shader transforms instance data into screen-space quads

### 4. TextEngine (`~200 lines`)

Two modes:

| Mode | Backend | Quality | Scope |
|------|---------|---------|-------|
| Browser | JS Canvas 2D via FFI | System text (emoji, CJK, shaping) | Full Unicode |
| Desktop | fontdue + swash | Latin/Cyrillic only | Fallback for offline |

```rust
pub enum TextBackend {
    BrowserCanvas,  // JS FFI -> OffscreenCanvas.measureText/fillText -> wgpu texture upload
    Fontdue,       // CPU rasterized glyphs -> glyph atlas texture
}
```

- Browser mode: text is rendered to an `OffscreenCanvas` via JS FFI, then uploaded to a wgpu texture atlas
- Fontdue mode: glyphs rasterized per-frame or cached in a texture atlas with LRU eviction
- Line wrapping: Taffy provides the available width; text engine lays out into that rect
- Result is a set of quads with texture coordinates into the glyph atlas

### 5. AssetCache (`~100 lines`)

```rust
pub struct AssetCache {
    textures: HashMap<String, wgpu::Texture>,
    fonts: HashMap<String, FontData>,
    images: HashMap<String, DecodedImage>,
}
```

- Texture upload via `wgpu::Queue::write_texture`
- Image decoding via `image` crate (optional, gated by `frontend` feature)
- Font loading via `fontdue` for desktop, browser fonts via JS FFI

### 6. InputMapper (`~150 lines`)

Normalizes platform input into a common Ifá-Lang event format:

| Platform | Source | Event type |
|----------|--------|------------|
| Desktop | winit::event::WindowEvent | `{type: "key", key: "Enter", ...}` |
| Browser | DOM Event → JS FFI | Same format |
| Terminal | crossterm::event | Same format (existing) |

```rust
pub enum UiEvent {
    Click { id: String, x: f32, y: f32 },
    Key { key: String, modifiers: KeyMods },
    Scroll { id: String, delta: f32 },
    Resize { width: f32, height: f32 },
    Hover { id: String, entered: bool },
    Custom { name: String, data: HashMap<String, IfaValue> },
}
```

- Hit testing: ray-cast input coordinates against the Taffy-computed rectangle tree
- Events bubble up the scene graph (similar to DOM event propagation)
- Ifá-Lang handlers are called via `ajose_signal.set(event)` or direct callback

### 7. Renderer (`~250 lines`)

```rust
pub struct UiRenderer {
    surface: wgpu::Surface,
    swapchain: wgpu::SwapChain,
    batcher: QuadBatcher,
    text_engine: TextEngine,
    pipeline: wgpu::RenderPipeline,
}
```

- One render pass per frame: clear → draw quads → draw text → draw Three.js viewport textures → present
- Pipeline state: blend mode (premultiplied alpha), depth test disabled (sorted instead), multisampling optional
- Frame lifecycle: acquire swapchain texture → create `Opon` epoch → mutate scene → layout → batch → encode → submit → present → truncate epoch

---

## Ose Domain Extension

In `crates/ifa-std/src/odu/ose.rs`, a new feature-gated dispatch arm:

```rust
#[cfg(feature = "webgpu-ui")]
"frontend" | "ojú_ọna" => Self::handle_frontend(args, ctx),
```

The handler creates an `UiRenderer`, enters a frame loop, and yields control to the event loop:

```rust
fn handle_frontend(args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
    let config = parse_frontend_config(&args);
    let renderer = UiRenderer::new(&config)?;
    let scene = SceneGraph::new();
    let layout = LayoutEngine::new();

    ctx.enter_event_loop(move |event| {
        scene.apply_ajose_mutations();
        layout.compute(&scene);
        let commands = batcher.batch(&scene, &layout);
        renderer.render(&commands);
    });

    Ok(IfaValue::null())
}
```

New methods added to Ose dispatch:

| Ifá name | English | Signature | Purpose |
|----------|---------|-----------|---------|
| `ojú_ọna` | `frontend` | `(config) -> nil` | Create UI window and enter render loop |
| `ya_ojú` | `draw` | `(frontend, scene_map) -> nil` | Push scene tree to renderer (frame update) |
| `gbé_sókè` | `raise` | `(frontend, event) -> nil` | Inject synthetic event |
| `fi_aworan` | `snapshot` | `(frontend) -> bytes` | Capture current frame as PNG |

---

## `ifafrontend.ifa` — Declarative Ifá-Lang API

A module in the standard library (`std/frontend/ifafrontend.ifa`) providing a declarative DSL:

```ifa
import "std/frontend/ifafrontend"

let app = Frontend.create(
    title: "My App",
    mode: "browser",    // "terminal" | "desktop" | "browser"
    width: 1024,
    height: 768,
)

let count = Signal(0)

app.render(fn ->
    Column(gap: 16, padding: 24, children: [
        Text("Count: {count.get()}", size: 32, weight: "bold"),
        Row(gap: 8, children: [
            Button(text: "-", variant: "primary", onclick: fn ->
                count.set(count.get() - 1)
            ),
            Button(text: "+", variant: "primary", onclick: fn ->
                count.set(count.get() + 1)
            ),
        ]),
        ThreeViewport(id: "model", height: 400),
    ])
)

// Three.js via JS FFI
let three = JSBridge.import("three")
// ... (see Three.js integration section)
```

The DSL provides:

| Component | Properties | Notes |
|-----------|------------|-------|
| `Column` | gap, padding, children | Flex column |
| `Row` | gap, padding, children | Flex row |
| `Text` | size, weight, color, align | Auto-escaping |
| `Button` | text, variant, onclick, disabled | Press state |
| `Image` | src, width, height | URL or data |
| `Input` | placeholder, value, oninput | Text entry |
| `ThreeViewport` | id, width, height | Three.js embed |
| `Canvas` | id, width, height | Raw canvas FFI |
| `Container` | style, onclick, children | Generic div |
| `Scrollable` | direction, children | Scroll container |

---

## Three.js Integration

Three.js (r171+, WebGPU production-ready since 2025) integrates via the JS FFI bridge. The recommended model is **texture embed**:

```
ifafrontend scene graph (2D UI)
    │
    └── ThreeViewport node ──► JS FFI bridge
                                    │
                                    └── Three.js scene
                                            │
                                    OffscreenCanvas
                                            │
                                    wgpu::Texture (uploaded)
                                            │
                                    QuadBatcher draws as a textured quad
```

In Ifá-Lang:

```ifa
let three = JSBridge.import("three")

let scene = three.Scene()
let camera = three.PerspectiveCamera(75, 1.333, 0.1, 1000)
let renderer = three.WebGPURenderer({
    canvas: JSBridge.getOffscreenCanvas("viewport"),
    antialias: true,
})

scene.add(three.GLTFLoader().load("assets/model.glb"))

app.on_tick(fn ->
    renderer.render(scene, camera)
)
```

Three.js is loaded at runtime via the FFI bridge. ifafrontend never ships 3D code — it provides a `ThreeViewport` node that reserves screen area, and the Three.js renderer's output is composited as a texture.

| Model | Overhead | Best for |
|-------|----------|----------|
| Texture embed | One texture upload per frame | 3D viewer in a UI card |
| Canvas overlay | Zero compositing overhead | Game with 3D main content |
| Shared wgpu device | Max performance, high complexity | Not recommended |

---

## Ajose Reactive Integration

Signals defined in `ifafrontend.ifa` are bridged to the Rust engine via a subscription registry:

```rust
pub struct ReactiveBridge {
    // Ifá-Lang signal ID -> Rust closure to update scene node
    subscriptions: HashMap<SignalId, Box<dyn Fn(&IfaValue) -> SceneMutation>>,
    pending_mutations: Vec<SceneMutation>,
}
```

When an Ifá-Lang `Signal.set()` fires:
1. The Ajose runtime calls the registered Rust-side effect
2. The effect produces a `SceneMutation` (e.g., `SetProperty { id, key: "color", value: "#ff0" }`)
3. Mutations are batched and applied to the scene graph before the next layout computation
4. Only changed nodes are re-laid-out (Taffy incremental compute)

This avoids V-DOM diffing entirely — the reactive graph is the diff.

---

## Opon Memory Per Frame

Each frame's render data lives in a dedicated Opon slot:

```
Frame N:
  Opon slot created → allocate scene graph, batcher instances, texture temporaries
  ─────────────────────────────────────────
  Process input events
  Apply Ajose mutations to scene graph
  Layout (Taffy)
  Batch quads
  Render (wgpu command encoding)
  Submit to GPU
  ─────────────────────────────────────────
  Opon slot truncated → bulk-free all frame data
```

- Persistent nodes (window, root layout) live outside the per-frame slot
- Transient data (instance buffers, temporary strings, layout caches) are bulk-freed
- Long-lived assets (textures, fonts) use `IfaGc` or `Arc` and are unaffected by epoch truncation

---

## Disadvantages & Mitigations

| # | Disadvantage | Mitigation |
|---|-------------|------------|
| 1 | **Text rendering** — GPU text needs glyph rasterization or browser FFI | Browser: delegate to Canvas 2D via JS FFI (native quality). Desktop: fontdue for Latin/Cyrillic, swash for advanced shaping. |
| 2 | **Accessibility** — GPU-rendered UI is opaque to screen readers | Browser mode: shadow DOM overlays with ARIA attributes. Desktop: accesskit crate integration. |
| 3 | **Two rendering engines** — terminal (ratatui) and GPU (wgpu) diverge | Shared Taffy layout engine. Only the render pass differs. Style grammar is identical. |
| 4 | **Web platform loss** — No native DOM events, forms, history, SEO | Selective DOM overlays for forms. SSR via ratatui output. Preserve non-DOM path for desktop/terminal. |
| 5 | **Ecosystem competition** — Not React/Flutter/Vue | Positioned as "bridge not replacement." JS FFI access to any ecosystem library. Unique differentiator: three output modes, epoch memory, fine-grained reactivity. |
| 6 | **Dual-runtime FFI cost** — JSON serialize/deserialize on every JS FFI call | Bridge uses JSON only (no shared memory, no serde). Acceptable because UI calls are infrequent (events, frame updates). Heavy data (textures) uses zero-copy canvas transfer. |
| 7 | **Mobile out of scope** — No iOS/Android native widgets | Mobile target via browser mode (PWA). Desktop wgpu only. |
| 8 | **wgpu churn** — WGSL and wgpu API evolve rapidly | Pin wgpu version. Abstract pipeline creation behind `Renderer` trait. Document migration path. |
| 9 | **CSS scope** — Cannot match full CSS spec | Intentionally bounded style grammar: layout (flexbox via Taffy) + 6 visual uniforms (color, bg, border, radius, shadow, opacity) + 4 text properties (size, weight, family, align). Escape hatch: direct wgpu buffer access or JS FFI. |
| 10 | **Developer ergonomics** — No hot-reload, no devtools | VM snapshot hot-reload (planned). Scene graph inspector via `dbg(ui_tree)` in Babalawo. |

---

## What Ifafrontend Cannot Do

| Feature | Reason |
|---------|--------|
| Mobile native widgets | No iOS/Android backend. Use PWA or browser mode. |
| OS-native widgets (file picker, print dialog) | Use JS FFI to call browser APIs, or wgpu escape hatch to platform APIs. |
| Full CSS (grid, animations, media queries) | Intentionally bounded. Grid → Taffy grid (planned). Animations → Ajose signal interpolation. Media queries → per-platform style constants. |
| Browser extension interop | Extensions operate on DOM; ifafrontend is a canvas. Selective DOM overlays for extension-sensitive content. |
| SEO without SSR | SSR via ratatui output. Ifafrontend outputs HTML strings in SSR mode (via existing `Element::render_to_string`). |
| Find-in-page | Browser's native find-in-page doesn't index canvas text. Mitigation: shadow DOM overlay with all visible text. |
| Complex text layout (bidirectional, vertical scripts) | Fontdue handles Latin/Cyrillic only. Arabic/Hebrew/CJK rely on browser FFI. |
| SVG rendering | Pass SVG to JS FFI → parse via browser → render to canvas → upload texture. |
| 3D rendering | Out of scope. Three.js via JS FFI bridge. |
| Instant first paint | wgpu requires adapter enumeration + device creation (~100-500ms). Warm-start cache mitigates. |
| Password manager autofill | Canvas is opaque to password managers. Mitigation: input fields rendered as DOM overlay for sensitive fields. |

---

## Integration Points

### With `ifa-infra/src/gpu.rs`

The existing `GpuContext` provides compute pipelines and memory pools. Ifafrontend adds:

- `Surface` + `SwapChain` creation (windowed output)
- `RenderPipeline` for quad rendering (vertex + fragment shader)
- `Texture` upload helpers (for canvas → texture transfer)

No changes to existing compute API — ifafrontend creates its own `wgpu::RenderPipeline` alongside the cached compute pipelines.

### With `ifa-infra/src/shaders.rs`

New WGSL shaders:

| Shader | Lines | Purpose |
|--------|-------|---------|
| `QUAD_VERTEX_SHADER` | ~40 | Instance data → clip-space quads |
| `QUAD_FRAGMENT_SHADER` | ~60 | Solid color / texture / rounded corners / border |
| `TEXT_GLYPH_SHADER` | ~30 | Glyph atlas sampling (SRGB) |

### With `crates/ifa-wasm`

WASM-specific surface creation:

```rust
#[cfg(target_arch = "wasm32")]
pub fn create_surface_from_canvas(
    instance: &wgpu::Instance,
    canvas_id: &str,
) -> wgpu::Surface {
    let canvas = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(canvas_id)
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into().unwrap();
    instance.create_surface_from_canvas(canvas).unwrap()
}
```

### With `crates/ifa-std/src/ffi.rs`

The JS FFI bridge (`boa_engine`, 1884 lines) is used for:
- Three.js control (scene manipulation, rendering)
- Canvas 2D API for text rendering
- DOM event forwarding (mouse, keyboard, touch)
- Asset loading (fetch images, fonts)
- OffscreenCanvas creation for Three.js and text

---

## Implementation Phases

| Phase | What | Files | Est. Lines |
|-------|------|-------|------------|
| 1 | GPU pipeline: `RenderPipeline` + `Surface` + `SwapChain` in `gpu.rs` | `crates/ifa-infra/src/gpu.rs`, `shaders.rs` | 300 |
| 2 | Layout: Taffy integration in `stacks/frontend.rs` | `stacks/frontend.rs` | 100 |
| 3 | Quad batcher: instanced rendering | `stacks/frontend.rs` | 400 |
| 4 | Text engine: browser FFI + fontdue | `stacks/frontend.rs` | 200 |
| 5 | Ose dispatch: `#[cfg(feature = "webgpu-ui")]` methods | `crates/ifa-std/src/odu/ose.rs` | 200 |
| 6 | WASM surface wiring | `crates/ifa-wasm/src/lib.rs` | 150 |
| 7 | `ifafrontend.ifa`: declarative module | `stacks/frontend/ifafrontend.ifa` | 500 |
| 8 | Input mapper + event loop | `stacks/frontend.rs` | 250 |
| 9 | Scene graph + Ajose bridge | `stacks/frontend.rs` | 400 |
| 10 | Asset cache (images, fonts) | `stacks/frontend.rs` | 150 |
| 11 | Three.js integration demo | Example `.ifa` file | 100 |

---

## Dependencies

New crates required in `stacks/frontend.rs`:

| Crate | Version | Purpose |
|-------|---------|---------|
| `taffy` | 0.5+ | Flexbox layout |
| `fontdue` | 0.9+ | Glyph rasterization (desktop fallback) |
| `image` | 0.25+ | Image decoding (optional) |
| `wgpu` | (already in `ifa-infra`) | GPU rendering |
| `winit` | 0.30+ | Native window creation |
| `accesskit` | 0.14+ | Desktop accessibility |

---

## File Manifest

```
stacks/frontend.rs                    — Existing: V-DOM/SSR/Router/Store
                                        To extend: +SceneGraph, +LayoutEngine,
                                        +QuadBatcher, +TextEngine, +InputMapper,
                                        +AssetCache, +Renderer, +ReactiveBridge

stacks/frontend/ifafrontend.ifa       — New: declarative Ifá-Lang DSL module (phase 7)

crates/ifa-std/src/odu/ose.rs         — Extend: +handle_frontend dispatch (phase 5)

crates/ifa-infra/src/gpu.rs           — Extend: +Surface/SwapChain/RenderPipeline (phase 1)

crates/ifa-infra/src/shaders.rs       — Extend: +quad vertex/fragment WGSL (phase 1)

crates/ifa-wasm/src/lib.rs            — Extend: +canvas surface creation (phase 6)

docs/design/ifafrontend.md            — This file
```
