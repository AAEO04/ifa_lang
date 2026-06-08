# Framework Architecture & Integration Strategy

Design and implement idiomatic Ifá-Lang frameworks (`.ifa` standard libraries) that wrap the underlying Rust native `stacks` (Backend, Frontend, ML, IoT, Gamedev) into developer-friendly, memory-safe APIs.

This plan outlines the design of these frameworks, borrowing aggressively from the best modern languages (Rust, Go, Python, Svelte, SolidJS) to ensure Ifá-Lang's standard libraries are state-of-the-art.

---

## 1. How the Ifá Libraries Integrate with Existing Rust Code
The `.ifa` framework files will act as **Zero-Overhead Orchestrators**. 
The Rust code (`backend.rs`, `iot.rs`, `cpu.rs`) handles the memory-safe native execution. The Ifá `.ifa` libraries will expose elegant, object-oriented or functional APIs that developers actually interact with. 

For example, when an Ifá developer calls `button.on_click(closure)`, the `.ifa` framework registers the closure in a generic VM list, but calls the native Rust `Ose.gboran()` (listen) under the hood to actually interface with the OS event loop. The Ifá library is pure sugar and architecture, while Rust handles the metal.

## 2. Framework Architectures

### Backend Framework: `IfaWeb`
Wraps `ifa-std`'s `backend` stack. Expands `Otura` natively with **tokio Async TCP/UDP**, **WebSockets**, and **TLS/SSL**.

**Stolen from Node/Express/Rails/FastAPI — grounded in actual Ifá-Lang syntax:**

- **Express-style Middleware (`server.use`)**: Composable pipeline. Consumed `yanda` bodies are statically tracked by `Babalawo` to prevent double-read.
- **FastAPI-style Auto-Validation**: `Babalawo` reads type annotations on route parameters and generates JSON Schema validation + interactive OpenAPI docs automatically.
- **Rails-style Active Record ORM (`Awo.Model`)**: Annotated `odu` classes auto-map to DB tables. `.save()`, `.find()`, `.where()` are convention-driven.
- **Native Async (`daro`/`reti`)**: Routes are async functions. The VM scheduler runs them on `tokio` without blocking.
- **Error Propagation (`?`)**: Route handlers can propagate errors up with the postfix `?` operator instead of nested try/catch.
- **Pipeline Operator (`|>`)**: Chain request transformations cleanly (parse → validate → store).
- **Null-Coalescing (`??`)**: Safe defaults on missing request fields without `if` clutter.
- **`defer` for Cleanup**: DB connections and open file handles are guaranteed to close at route scope exit.
- **`match` for Status Codes**: Pattern match on result types to build ergonomic response dispatch.

```ifa
iba IfaWeb lati "std:backend";
iba Awo lati "std:orm";

// Rails-style Active Record model (odu class with type hints)
gbangba odu User < Awo.Model {
    ayanmo oruko: Str;
    ayanmo ojo_ori: Int;
    ayanmo imeeli: Str;
}

// FastAPI-style validated async route — Babalawo enforces req:## Phase 2: Structural and Academic Grade Improvements

The previous phase addressed the critical runtime bugs (GC thread safety, Sandbox PID, and FFI shadowing) and implemented runtime `Ewo` asserts. The Academic Grade Report also highlighted several structural and compiler-level deficiencies. Based on your request to "check ikin and the likes" and address "the other things mentioned in the report", here is the proposed plan for Phase 2:

### User Review Required

> [!WARNING]
> **Scoping the Compiler Fixes**
> The report notes the lack of an Intermediate Representation (IR), optimizations, and abstract interpretation. Building a full IR (like SSA) and an optimization pipeline is a massive, multi-week undertaking that would require fundamentally rewriting `ifa-compiler`. Do you want me to start laying the groundwork for a basic IR, or should we focus on the lower-hanging fruit first (Spans, Validation, Ikin)?

### Proposed Changes

#### 1. Ikin (Constant Pool) Enhancements
- Review `crates/ifa-vm/src/vm_ikin.rs` to ensure string deduplication and intern limits (`MAX_INTERNED_STRINGS`) are robust.
- Add safeguards against constant pool exhaustion or optimize the memory footprint of the flash nuts for embedded environments.

#### 2. AST Spans and Error Quality (`ifa-parser`)
- The report states: *"Expressions lack Span fields (critical for error quality)."*
- **Action:** Modify the AST structures in `ifa-parser` (specifically `Expression` nodes) to include byte-range Spans from the Logos/Pest tokens. Update `ifa-babalawo` to use these spans so that static analysis errors point to the exact column/line instead of just the file.

#### 3. Bytecode Validator (`ifa-bytecode`)
- The report states: *"no instruction validator"*
- **Action:** Introduce a validation pass in `ifa-bytecode` that runs before execution. It will statically verify that jumps (`Jump`, `BranchTrue`, `BranchFalse`) do not target out-of-bounds instruction offsets, preventing segfaults or panics during `ifa-vm` execution.

#### 4. Cross-Module Resolution (`ifa-babalawo`)
- The report states: *"Babalawo can't verify that a function exists in an imported module"*
- **Action:** Enhance `ifa-babalawo` to perform basic multi-pass symbol resolution. When an `ibà` (import) is declared, the analyzer should resolve the exported symbols of the target file to ensure function calls to imported domains actually exist.

### Verification Plan
- **Automated Tests**: Write tests for the bytecode validator to ensure it catches malformed `.ifab` files. Add tests for `Ikin` deduplication.
- **Manual Verification**: Run `ifa check` on a project with multiple files to verify cross-module function resolution and inspect the new span-accurate error messages. time
gbangba daro ese create_user(req: User) {
    defer { Awo.close_connection(); }

    // Pipeline: parse → validate → save. Error propagation via ?
    ayanmo user = yanda req
        |> Awo.validate(?)
        |> User.save(?);

    // match on result type
    match (user) {
        Awo.Ok(u)  => pada IfaWeb.Response(201, u);
        Awo.Err(e) => pada IfaWeb.Response(422, e.message);
    }
}

// Middleware chain (Express-style)
gbangba daro ese auth_middleware(req, next) {
    ayanmo token = req.headers["Authorization"] ?? "";
    gbiyanju {
        reti IfaWeb.verify_jwt(token)?;
        pada reti next(req);
    } gba (e) {
        pada IfaWeb.Response(401, "Unauthorized");
    }
}

ayanmo server = IfaWeb.Server("0.0.0.0:8080");
server.use(IfaWeb.logger);       // Express: req logging
server.use(auth_middleware);     // Express: JWT auth
server.post("/users", create_user);
reti server.listen();
```

### Frontend Framework: `IfaFrontend`
Wraps `stacks/frontend.rs` (`Element`, `Node`, `Router`, `Store`, `Fetch`, `LocalStorage`).

**Odù Backends**: `Ika` (HTML escaping/SSR), `Osa` (async fetch), `Irete` (HTTPS), `Ose` (WASM canvas)

**Stolen Ideas & Real APIs**:
- **Server-Side Rendering (SSR) via `Element.render()`**: `frontend.rs` already has a full `Element` builder with `.render_to_string()`. We will wire `IfaWeb` + `IfaFrontend` together so routes can render full HTML pages on the server before sending to the client.
- **SolidJS fine-grained `Store<T>`**: The `Store<T>` struct already supports `.get()`, `.set()`, and `.subscribe(callback)`. We will expose this as a reactive signal primitive in Ifá-Lang.
- **Vue-style component DSL**: Each `.ifa` component file is one `ese` function returning an `Element` tree built with the real hyperscript helpers (`div()`, `button()`, `input()`, `form()`, etc.).
- **Client Fetch (`Fetch.get/post`)**: The `Fetch` + `FetchBuilder` chain already handles HTTP. Exposed via `daro`/`reti` in Ifá-Lang.
- **Client State (`LocalStorage`)**: Wraps browser `localStorage` for persistence across page reloads.

```ifa
iba IfaFrontend lati "std:frontend";

// Vue-style component: encapsulates state + view in one ese
gbangba ese Counter() {
    // Store<T> from frontend.rs — reactive state
    ayanmo state = IfaFrontend.Store(0);

    // Real hyperscript helpers from frontend.rs
    pada IfaFrontend.div()
        .class("card")
        .child(
            IfaFrontend.button()
                .text($"Clicks: {state.get()}")
                .on("click", ese() {
                    state.set(state.get() + 1);
                })
        )
        .render();
}

// Server-side: render to HTML string for IfaWeb route
gbangba ese page_handler(req) {
    ayanmo html = Counter();
    pada IfaWeb.Response(200, html);
}

// Async fetch from the browser (daro/reti)
gbangba daro ese load_users() {
    ayanmo resp = reti IfaFrontend.Fetch.get("/api/users")
        .header("Accept", "application/json")
        .send()?;
    pada resp.json();
}
```

### Machine Learning Framework: `IfaML`
Wraps `stacks/ml.rs` (`Tensor`, `Linear`, `SGD`, loss functions, `to_gpu`).

**Odù Backends**: `Obara` (math), `Oturupon` (precision arithmetic), `Cpu` + `Gpu` infra (parallel compute)

**Real APIs from `ml.rs`** + **Stolen Ideas**:
- **Full Tensor API**: `Tensor::zeros`, `ones`, `rand`, `randn`, `reshape`, `transpose`, `flatten`, `matmul`, `relu`, `sigmoid`, `tanh`, `softmax`, `log_softmax`, `clamp`, `argmax`.
- **GPU Acceleration via `to_gpu(ctx)`**: `ml.rs` already has `Tensor::to_gpu(&GpuContext)`. We wire this to `ifa-infra`'s `GpuContext` and `MemoryPool` (slab allocator) for zero-per-allocation GPU tensors.
- **JAX-style `grad(closure)`**: Since we control the VM bytecode, we can trace a pure closure's arithmetic ops and automatically generate the reverse-mode gradient as a new function.
- **Backend-Agnostic (Rust Burn/Candle)**: The same `.ifa` code runs on CPU (Rayon `par_map`/`matmul`) or GPU (`wgpu` dispatch) based on a runtime backend flag. The `Tensor::to_gpu()` call is the explicit promotion point.
- **Memory Safety (RAII `lo` blocks)**: Tensors dropped at lexical scope exit — no GC cycle needed. `MemoryPool` slab allocator handles same-shape tensor reuse.
- **Static Shape Checking via Babalawo**: Shape annotations on `Tensor` calls let `Babalawo` reject dimension mismatches at `ifa check` time.

```ifa
iba IfaML lati "std:ml";

// Real Tensor API from ml.rs
ayanmo a = IfaML.Tensor.from_2d([[1.0, 2.0], [3.0, 4.0]]);
ayanmo b = IfaML.Tensor.rand([2, 2]);

// RAII: tensor and GPU buffer freed at lo-block exit, not at GC cycle
lo (ayanmo gpu_a = a.to_gpu(Gpu.init())?) {
    ayanmo result = Gpu.matmul(gpu_a, b, 2, 2, 2);
    Irosu.fo($"mean: {result.mean()}");
}

// Neural net with real Linear layer from ml.rs
ayanmo layer = IfaML.Linear.new(784, 128);
ayanmo out = layer.forward(a)?;
ayanmo loss = IfaML.mse(out, IfaML.Tensor.zeros([2, 2]));

// JAX-style: auto-differentiate a pure Ifá-Lang function
ese square(x) { pada x * x; }
ayanmo d_square = IfaML.grad(square);
Irosu.fo(d_square(3.0)); // prints 6.0

// Optimiser
ayanmo opt = IfaML.SGD.new(0.01);
```

### Game Development & Windowing: `IfaGame` / `Ifa3D`
Wraps `stacks/gamedev.rs` (`World`, `Entity`, `Transform`, `Velocity`, `Collider`, `SpriteComponent`, `SparseSet`, `SpatialGrid`, `Input`, `GameTimer`, `Animation`, `Audio`, `Vec2`, `AABB`).

**Odù Backends**: `Ose` (wgpu render + windowing), `Osa` (parallel ECS systems), `Owonrin` (procedural gen), `Iwori` (delta-time), `Obara` (physics math)

**Real APIs from `gamedev.rs`** + **Stolen Ideas**:
- **Full ECS with `SparseSet<T>`**: Sparse set component storage — O(1) insert/get/remove. Components (`Transform`, `Velocity`, `Collider`, `SpriteComponent`) stored in typed sparse sets, not GC heap objects. This resolves Linus's "fake ECS" critique.
- **Spatial Partitioning (`SpatialGrid`)**: Built-in broad-phase collision grid with `query(AABB)` for O(1) neighbourhood lookups.
- **Input (`Input.is_key_down`, `is_key_pressed`)**: Frame-accurate keyboard state.
- **Timer (`GameTimer.once/repeating`, `.tick(dt)`)**: Event timers with `.progress()` for animations.
- **Animation**: Frame-based sprite animation with configurable FPS and looping.
- **Audio**: `play_sound`, `play_music`, `set_volume` via Rodio (when `audio` feature enabled).
- **Three.js-style `Ifa3D`**: High-level Scene Graph API (`Scene`, `PerspectiveCamera`, `Mesh`, `BoxGeometry`, `StandardMaterial`) backed by `Ose` wgpu pipelines + pre-compiled WGSL shaders from `ifa-infra/src/shaders.rs`.

```ifa
iba IfaGame lati "std:gamedev";

// Real ECS from gamedev.rs
ayanmo world = IfaGame.World.new();
ayanmo player = world.spawn();
world.add_transform(player, IfaGame.Transform.new(0.0, 0.0));
world.add_velocity(player, IfaGame.Velocity{dx: 5.0, dy: 0.0});
world.add_collider(player, IfaGame.Collider{width: 32.0, height: 32.0});
world.add_tag(player, "player");

// Spatial grid broad-phase
ayanmo grid = IfaGame.SpatialGrid.new(64.0);
ayanmo nearby = grid.query(IfaGame.AABB.new(0.0, 0.0, 128.0, 128.0));

// Frame-accurate input
ayanmo input = IfaGame.Input.new();
ti input.is_key_down("ArrowRight") {
    world.transform_mut(player)?.x += 2.0;
}

// Animation
ayanmo anim = IfaGame.Animation.new(["walk_0", "walk_1", "walk_2"], 12.0, otito);
ayanmo timer = IfaGame.GameTimer.repeating(1.0 / 60.0);

// Audio
ayanmo audio = IfaGame.Audio.new()?;
audio.play_music("assets/bg.ogg", otito)?;

// Ifa3D scene graph backed by Ose wgpu
ayanmo scene = Ifa3D.Scene();
ayanmo camera = Ifa3D.PerspectiveCamera(75.0, 16.0/9.0, 0.1, 1000.0);
ayanmo cube = Ifa3D.Mesh(
    Ifa3D.BoxGeometry(1.0, 1.0, 1.0),
    Ifa3D.StandardMaterial({"color": "#ff0000"})
);
scene.add(cube);
Ifa3D.render(scene, camera);
```

### IoT & Embedded: `IfaIoT`
Wraps `stacks/iot.rs` (`GpioPin`, `EmbeddedGpio`, `EmbeddedTimer`, `EmbeddedSerial`, `EmbeddedI2C`, `EmbeddedSPI`, `SensorReading`, `flash`).

**Odù Backends**: `Odi` (flash storage), `Iwori` (timer/delay), `Irosu` (serial I/O), `Ofun` (capability gate — IoT requires explicit `Hardware` capability)

**Real APIs from `iot.rs`** + **Value-Add**:
- **GPIO (`GpioPin`, `EmbeddedGpio`)**: `set_high/low/toggle`, `read`, `is_high`, `pwm`, `analog_read` — full digital + analog pin control.
- **Timer (`EmbeddedTimer`)**: `delay_us`, `delay_ms`, `start`, `is_expired`, `wait`, `measure(closure)` for nanosecond profiling.
- **Serial UART (`EmbeddedSerial`)**: `init(baud)`, `write`, `print`, `read`, `available`.
- **I2C (`EmbeddedI2C`)**: `init(sda, scl)`, `write(addr, data)`, `read(addr, buf)`, `write_read` — covers most sensor protocols (SSD1306, MPU6050, BME280).
- **SPI (`EmbeddedSPI`)**: `init(mosi, miso, sck)`, `transfer`, `write` — for high-speed peripherals.
- **Flash firmware (`flash(target, binary, port)`)**: Deploy compiled Ifá-Lang embedded binary directly to ESP32/STM32/RP2040.
- **Value-Add**: The Ifá-Lang framework layer adds declarative event loops over the raw HAL. Instead of a `nigba` polling loop, we expose `on_threshold` and `on_change` patterns.

```ifa
iba IfaIoT lati "std:iot";

// GPIO: control an LED on pin 4
ayanmo led = IfaIoT.GpioPin.new(4);
led.set_mode(IfaIoT.PinMode.Output)?;
led.set_high()?;

// Timer: precise microsecond delay
ayanmo timer = IfaIoT.EmbeddedTimer.new();
timer.delay_ms(500);
led.toggle()?;

// I2C: write to SSD1306 OLED display at address 0x3C
ayanmo i2c = IfaIoT.EmbeddedI2C.new();
i2c.init(21, 22)?; // SDA=21, SCL=22 (ESP32 default)
i2c.write(0x3C, [0x00, 0xAE])?; // display off command

// Serial: debug output over UART
ayanmo serial = IfaIoT.EmbeddedSerial.new();
serial.init(115200)?;
serial.print("IfaIoT boot OK");

// Analog sensor read on pin A0
ayanmo gpio = IfaIoT.EmbeddedGpio;
ayanmo raw = gpio.analog_read(0)?;
ayanmo voltage = raw.to_float() * 3.3 / 4095.0;

// Declarative threshold event (value-add layer)
esa({ gpio.analog_read(0)? > 2048 }, ese() {
    led.set_high()?;
    serial.print("Threshold breached!");
});
```

---

## 3. Tooling Integration & Architecture

### Ika HTML vs IfaFrontend
It's important to delineate the separation of concerns:
- **`Ika` (Domain 9)**: Uses the Rust `tl` crate for Server-Side Parsing (like BeautifulSoup/Cheerio). It is strictly used in backend contexts (`IfaWeb`) to parse raw HTML strings, strip tags, or perform query selections (`wa_html`).
- **`IfaFrontend`**: Does *not* use `Ika`. It runs client-side (via WASM) and interacts directly with browser memory using `web_sys` to mathematically manipulate DOM nodes without string parsing.

### How Libraries Plug into the VM & Babalawo
Bridging `.ifa` framework code, native Rust execution, and the static analyzer requires two systems:

**A. VM Module Loader (`ifa-vm`)**
1. When the user writes `iba IfaFrontend lati "std:frontend";`, the VM intercepts the `std:` prefix.
2. The VM loads the framework's `.ifa` source code from an embedded binary string (compiled into the CLI via `include_str!`).
3. The VM executes this standard library code, binding the high-level classes/signals to the native `Registry` Odù domains (like `Ose` or `Otura`), and registers the resulting objects in the user's scope.

**B. Static Analysis (`Babalawo`)**
`Babalawo` must know that `IfaFrontend.div()` is valid *before* runtime.
1. **Intrinsic ASTs**: `Babalawo` will maintain pre-computed Abstract Syntax Trees (ASTs) or "Stub Files" (similar to TypeScript's `.d.ts`) for all `std:` frameworks.
2. **Typechecking**: Upon seeing an `iba` import, `Babalawo` merges these framework definitions into its symbol table, enabling strict shape and type checking on API calls without needing to boot the VM.

---

## 4. The 16 Odù Domains — Mapping to Framework Roles

Every framework in the Ifá-Lang ecosystem is backed by one or more of the 16 Odù domains implemented in Rust (`ifa-std/src/odu/`). Understanding this mapping is critical for knowing which Rust code to extend when adding features.

| # | Odù | Rust File | Domain Purpose | Consumed By |
|---|-----|-----------|----------------|-------------|
| 1 | **Ogbe** | `ogbe.rs` | Identity, reflection, introspection (`id`, `type_of`) | `Babalawo` type stubs |
| 2 | **Oyeku** | `oyeku.rs` | Process lifecycle (`ku`, `sun`, `duro`, `da_duro`) | `IfaWeb` graceful shutdown, `IfaIoT` timers |
| 3 | **Iwori** | `iwori.rs` | Time (`akoko`, `timestamp`, timers) | `IfaWeb` request timing, `IfaML` profiling |
| 4 | **Odi** | `odi.rs` | Filesystem (`ka`, `kọ`, `yi_oruko`, `paarẹ`) | `IfaWeb` static files, `IfaIoT` flash storage |
| 5 | **Irosu** | `irosu.rs` | Standard I/O (`fo`, `ka_ila`, `stdin`, `stdout`) | REPL, `IfaWeb` logging |
| 6 | **Owonrin** | `owonrin.rs` | Randomness (`pese`, `uuid`, `yan`, `dapo`, `boya`) | `IfaWeb` token generation, `IfaGame` procedural gen |
| 7 | **Obara** | `obara.rs` | Math (`fikun`, `mu`, `square_root`, trigonometry) | `IfaML` activations, `Ifa3D` transforms |
| 8 | **Okanran** | `okanran.rs` | Assertions & testing (`beeni`, `dogba`, `ta`, `ku`, `gbiyanju`, `wo`) | `ifa test`, `IfaML` shape checks |
| 9 | **Ogunda** | `ogunda.rs` | Collections + Process spawn (`map`, `filter`, `sise`, `bere`, `ayika`) | `IfaWeb` env vars, shell scripts |
| 10 | **Osa** | `osa.rs` | Concurrency (`sa` spawn, `oju_ona` channels, `titipe` mutex, `kaka` rwlock) | `IfaWeb` async, `IfaGame` ECS systems |
| 11 | **Ika** | `ika.rs` | Strings, JSON, CSV, HTML parsing, Regex (`mo`, `wa_html`, `yi_si_json`) | `IfaWeb` SSR/parsing, `IfaFrontend` SSR |
| 12 | **Oturupon** | `oturupon.rs` | Precise integer/float arithmetic (`din`, `pin`, `ku`, `iyoku`) | `IfaML` tensor ops, `Ifa3D` geometry |
| 13 | **Otura** | `otura.rs` | Networking (to be expanded with tokio TCP/UDP/WS/TLS) | `IfaWeb` HTTP server, WebSockets |
| 14 | **Irete** | `irete.rs` | Cryptography (`hash`, `encrypt`, `sign`, `verify`) | `IfaWeb` JWT, TLS handshake |
| 15 | **Ose** | `ose.rs` | Graphics, TUI, GPU dispatch (`wgpu` render, windowing) | `Ifa3D`, `IfaGame`, `IfaFrontend` WASM canvas |
| 16 | **Ofun** | `ofun.rs` | Capability sandbox (permission gates: Network, Filesystem, GPU) | All frameworks — enforced at VM entry |

### Using Odù Directly in Ifá-Lang Code

Odù domains are first-class citizens and can be called directly using dot syntax. No `iba` needed:

```ifa
// Ogunda (Domain 9): spawn a shell process
ayanmo pid = Ogunda.bere("python3", ["script.py"]);

// Osa (Domain 10): create a typed channel for concurrent message passing
ayanmo (tx, rx) = Osa.oju_ona(100);
Osa.sa(ese() { tx.send("hello"); });

// Owonrin (Domain 6): generate a UUID and a random float
ayanmo id = Owonrin.uuid();
ayanmo roll = Owonrin.pese_larin(0.0, 1.0);

// Okanran (Domain 8): assert in tests — panics with Yoruba wisdom on failure
Okanran.dogba(result, expected);

// Osa mutex for shared mutable state across async routes
ayanmo state = Osa.titipe({"requests": 0});
```

---

## 5. Infra Support (`ifa-infra`)

The `ifa-infra` crate is the **bare-metal performance layer** — sitting below `ifa-std`. It provides low-level hardware abstractions that power `IfaML`, `Ifa3D`, and high-throughput `IfaWeb` workloads.

### CPU (`ifa-infra/src/cpu.rs`)
Parallel compute via Rayon. All operations are zero-allocation on the hot path.

| Function | Purpose | Used By |
|----------|---------|---------|
| `par_map`, `par_filter`, `par_reduce` | Parallel data transforms | `IfaML` batch inference |
| `matmul(a, b, m, n, k)` | BLAS-style matrix multiply on f32 slices | `IfaML` Linear layers |
| `dot`, `vec_add`, `vec_mul`, `scale_bias` | SIMD-friendly vector ops | `IfaML`, `Ifa3D` physics |
| `relu(data)` | In-place activation (mutates `&mut [f32]`) | `IfaML` neural networks |
| `TaskGraph` | DAG task scheduler with `add_dependency` and timeout | `IfaML` pipeline, `IfaGame` systems |
| `profile(name, fn)` | Wraps closures with nanosecond timing | `IfaML` JIT profiling |

```ifa
// Access the Cpu infra domain directly via the Cpu reserved Odu name
ayanmo result = Cpu.matmul(a_data, b_data, 512, 512, 512);
```

### GPU (`ifa-infra/src/gpu.rs`)
`wgpu`-backed compute context. Powers both `IfaML` tensor ops and `Ifa3D` rendering.

| Feature | Purpose |
|---------|---------|
| `GpuContext::new_blocking()` | Acquire `wgpu` adapter + device |
| `create_compute_pipeline(shader, entry)` | Compile and cache WGSL compute shader |
| `dispatch_pipeline(name, x, y, z)` | Fire compute workgroup on GPU |
| `MemoryPool` (arena allocator) | Pre-allocate large contiguous GPU buffers — avoids per-tensor allocation overhead flagged by Linus |
| `SlabMemoryPool` | Fixed-size slab allocator for same-shape tensors |
| `matmul`, `relu`, `vec_add`, `map_scale_bias` | Core GPU-accelerated ML primitives |

```ifa
// Gpu is a reserved Odu name — access directly
ayanmo ctx = Gpu.init();
ayanmo pool = Gpu.memory_pool(1024 * 1024 * 512); // 512MB arena
lo (ayanmo buf = pool.allocate(4096)) {
    Gpu.relu(buf, 1024);
} // Pool allocation returned at scope exit — no GC cycle needed
```

### Storage (`ifa-infra/src/storage.rs` — `OduStore`)
Persistent key-value store used for module caching and session state.

### Kernel (`ifa-infra/src/kernel.rs`)
Exposes `num_cores()`, `uptime()`, `total_memory()`, `available_memory()` for system monitoring in `IfaWeb` health endpoints and `IfaIoT` resource guards.

### Shaders (`ifa-infra/src/shaders.rs`)
Pre-compiled WGSL shaders for `Ifa3D`'s rendering pipeline (PBR materials, shadow maps).