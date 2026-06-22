# AGENTS.md

## Build & Run

```sh
cargo build                          # debug build
cargo run --release -- --mode graphics    # Vulkan colored cells (PRIMARY, recommended)
cargo run --release -- --mode ascii       # Vulkan ASCII glyphs (debug backend)
cargo run -- --mode terminal         # ANSI terminal mode (legacy)
cargo run -- --mode pipe             # JSON stdin/stdout for AI agents
cargo run -- --mode test             # run all JSON scenarios
cargo run -- --mode headless --headless-ticks 60  # dump to headless_dump.txt
cargo run -- --mode capture --headless-ticks 60    # render graphics-like PNG to capture.png
cargo run --release -- --mode benchmark --benchmark-ticks 600 --benchmark-renderer graphics --benchmark-biome surface  # FPS benchmark (surface)
cargo run --release -- --mode benchmark --benchmark-ticks 600 --benchmark-renderer graphics --benchmark-biome caves     # FPS benchmark (caves)
cargo run --release -- --mode benchmark --benchmark-ticks 600 --benchmark-renderer graphics --benchmark-biome dungeon  # FPS benchmark (dungeon)
cargo run --release -- --mode tape --headless-ticks 300 --tape-interval 10 --tape-output tape.txt --tape-json tape.json  # multi-spectrum recording
```

Rust edition 2024, requires rustc >= 1.96. Vulkan 1.2+ required for `ascii`/`graphics` modes (falls back to terminal). Use `--release` for playable frame rates; GPU modes are CPU-bound in debug builds due to the cellular-automaton simulation. The GPU event loop is capped at 60 FPS.

## Architecture Priorities

- **Graphics mode** (`--mode graphics`) is the PRIMARY renderer — colored cells, 8x8 px, 16:9 window
- **ASCII mode** (`--mode ascii`) is a DEBUG backend — same Vulkan pipeline with glyph atlas
- **Terminal mode** (`--mode terminal`) is LEGACY — kept for headless/test compatibility
- **AI observation** uses multi-spectrum ASCII layers via pipe protocol:
  - `materials` — material type per cell
  - `temperature` — heat levels encoded as characters
  - `light` — light intensity per cell
  - `entities` — entity positions and types only
  - `density` — material density visualization
  - `velocity` — entity movement speed and CA activity

## Tape System

```sh
# Record all spectrums every 10 ticks for 300 ticks
cargo run --release -- --mode tape --headless-ticks 300 --tape-interval 10 \
  --tape-output tape.txt --tape-json tape.json
```

Each tape frame contains:
- Tick number, depth, kills, score
- Camera position
- Player HP, position, entity count
- All 6 spectrum layers as ASCII text

Pipe protocol spectrum commands:
- `{"cmd":"get_spectrum","spectrum":"materials","w":80,"h":25}` — single spectrum
- `{"cmd":"get_all_spectrums","w":80,"h":25}` — all spectrums at once

## Tests

```sh
cargo test                           # all 185 integration tests (171 original + 14 multilayer)
cargo test --test physics_sand       # single test file
cargo test --test multilayer         # multi-layer world tests
cargo run -- --mode test --scenario-dir scenarios  # 14 JSON scenarios
```

Tests are in `tests/*.rs`, use `verbatim::ai::GameSession` API (not the render loop). Scenarios are in `scenarios/*.json`. Run `python3 tools/render_dump.py` to convert a `headless_dump.txt` frame to PNG for visual inspection. Use `--mode capture` to generate a graphics-like PNG directly without a window.

## Controls

Terminal (`--mode terminal`):
- `a` / `d` or `←` / `→` — move
- `w` / `↑` / `space` — jump (press only)
- `h` / `j` / `k` / `l` — shoot left / down / up / right
- `f` — toggle fireball mode
- `>` — descend when standing on stairs
- `e` — use/equip first inventory item
- `r` — drop first inventory item
- `1`–`0` / `x` — paint material brush
- `q` / `ctrl-c` — quit

GPU (`--mode ascii` / `--mode graphics`):
- `a` / `d` or `←` / `→` — move
- `w` / `↑` / `space` — jump
- `h` / `j` / `k` / `l` — shoot left / down / up / right (keyboard fallback)
- **Left-click** — shoot toward mouse cursor (Noita-style aiming)
- **Mouse position** — player faces mouse direction (left/right)
- `f` — toggle fireball mode
- `>` / `.` — descend when standing on stairs
- `e` — use/equip first inventory item
- `r` — drop first inventory item
- `1`–`0` / `x` — paint material brush
- `y` / `u` / `i` / `o` — move camera offset
- `m` — toggle audio
- `q` / `esc` — quit

## Shaders

GLSL shaders in `assets/shaders/`. Pre-compiled to SPIR-V via `glslangValidator -V`. Recompile after editing:

```sh
glslangValidator -V assets/shaders/cell.vert -o assets/shaders/cell_vert.spv
glslangValidator -V assets/shaders/cell.frag -o assets/shaders/cell_frag.spv
glslangValidator -V assets/shaders/graphics.vert -o assets/shaders/graphics_vert.spv
glslangValidator -V assets/shaders/graphics.frag -o assets/shaders/graphics_frag.spv
```

SPV files are committed. `include_bytes!` embeds them at compile time.

## Architecture

**Source of truth**: `ChunkedGrid` of `Cell` structs with parallel per-chunk layer arrays. Replaces the old fixed-size `Grid` with dual storage:

- **Bounded mode**: `Vec<Chunk>` for deterministic 250x250 test/AI grids.
- **Infinite mode**: `HashMap<(i64, i64), Chunk>` for continuous 12500x12500 cell (100000x100000 px) Noita-scale worlds.

Main game (`--mode terminal`, `--mode ascii`, `--mode graphics`) uses the infinite mode. Each `Cell` stores `material`, `fg`/`bg` color, `variant` inline (9 bytes, no temp). Temperature, gas, pressure, and light are stored in parallel arrays per chunk. Chunks are 64x64 cells with `active`, `modified`, `was_modified`, `generated`, and `dirty` flags.

**Multi-layer world** (Phase 6): Each `Chunk` has 5 parallel arrays alongside `cells`:
- `temps: Vec<f32>` — temperature per cell (16 KB/chunk)
- `pressure: Vec<u8>` — pressure per cell, 128 = atmospheric (4 KB/chunk)
- `gas_type: Vec<u8>` — gas type: 0=air, 1=smoke, 2=poison, 3=CO2, 4=steam (4 KB/chunk)
- `gas_density: Vec<u8>` — gas concentration 0-255 (4 KB/chunk)
- `light: Vec<[u8;3]>` — world-space RGB light (12 KB/chunk)

Layer access via `grid.get_temp()`/`set_temp()`, `grid.get_gas()`/`set_gas()`, `grid.get_pressure()`/`set_pressure()`, `grid.get_light()`/`set_light()`. All `set_*` methods call `mark_dirty()` (shared dirty rect). `cells_swap` swaps all layers. `set_material` also sets `default_temp()`.

**Simulation steps** per `fixed_update()`: `apply_gas_damage()` → `ca.step()` (material CA + heat_transfer + gas_step + pressure_step + light_step) → entity updates → combat → status effects. The `ca.step()` saves pre-clear dirty rects and passes them to layer steps so heat/gas/pressure can diffuse beyond CA-active cells.

**Serialization**: Multi-section chunk format with `VWM1` magic header. Sections: cells (8 bytes × 4096), temps (4 × 4096), gas (2 × 4096), pressure (1 × 4096), light (3 × 4096). Old 12-byte format auto-detected and loaded for backward compat.

**Four entity kinds**: `Player`, `Goblin`, `Slime`, `Corpse`. Three physics types: cellular (CA materials in grid), rigid (alive entities, AABB + slope stepping), ragdoll (corpses, Verlet constraints).

**Three render modes**: `terminal` (crossterm ANSI), `ascii` (Vulkan glyph atlas + instanced), `graphics` (Vulkan colored quads). All three read the same `ChunkedGrid` + `EntityManager` and an optional `LightGrid` overlay. GPU renderers upload a viewport-relative grid buffer and index it in shaders with `cam_pos`.

**Lighting pass**: `render::lighting::LightGrid` is computed each frame on the CPU. Light sources are emitted by `Lava` and `Fire` cells. Light attenuates with distance and is blocked by solid cells (ray-cast line-of-sight). The ambient light level is configurable per mode; the default ambient is `[100, 100, 120]`. The `Renderer` trait and all renderers accept `Option<&LightGrid>`; `UiLayer` elements are drawn unlit on top.

**GpuRenderer trait** (`main.rs`): unifies `VulkanRenderer` and `GraphicsRenderer` behind `run_gpu_mode<R>()`. Both have identical event loops; only shader/instance format differs.

**Game loop**: `Game::fixed_update()` = activate chunks -> CA step -> rigid update -> ragdoll update -> slime AI -> goblin AI -> combat -> projectiles -> damage -> corpse decomposition -> status effects -> score -> item pickup. Called from event loop with fixed 16ms accumulator. `Game::run()` is terminal-only (uses `InputHandler` with crossterm). GPU modes use `run_gpu_mode` with `WindowInput` (winit PhysicalKey, layout-agnostic).

**BodyTemplate** (`entity/body_template.rs`): data-driven entity body definitions. JSON-serializable. `build_humanoid()` delegates to `template_for_kind().apply_to()`. To add a new creature shape, add a `BodyTemplate` constructor + match arm in `template_for_kind`. Each `SubBody` has a `color: [u8; 4]` field for per-part coloring.


**Combat**: player uses ranged projectiles (Arrow / Magic Bolt / Fireball). `player_shoot()` fires in the direction held. `update_projectiles()` moves projectiles, resolves entity hits (damage + knockback), and applies fireball ignition. Enemy contact damage still applies (Goblin 8 dmg, Slime 5 dmg per 20 ticks) reduced by equipped armor. Knockback applied to player.

**Slime AI**: `update_slime_ai()` makes slimes jump toward player every 60 ticks when within 40 cells. Jump power scales with proximity.

**Goblin AI**: `update_goblin_ai()` moves toward player when far, backs away when close, and flees when health < 10.

**Corpse decomposition**: `decompose_corpses()` turns dead `Corpse` bodies into `Flesh` cells in the grid over time.

**RPG layer**: `Entity` carries stats (strength, agility, toughness, willpower), level, XP, status effects (on_fire/poisoned/frozen/bleeding), and `add_xp` / `xp_to_level` / `recalc_max_health`. Status effects deal damage or expire in `update_status_effects`. `EntityInfo` exposes these for AI state.

**Items & inventory**: `Item` (weapon, armor, consumable), `ItemManager`, and inventory/equipment slots on `Player`. Items spawn in the world and are picked up on contact. `Game::use_item(0)` equips weapons/armor or consumes potions; `Game::drop_item(0)` returns an item to the world. Equipped weapon adds damage bonus to projectiles; equipped armor reduces enemy contact damage.

**UI layer**: `ui::UiLayer` overlays non-destructive UI on all renderers. Health bars above entities, bottom-line HUD, scrolling message log, floating damage numbers, screen-edge indicators, death screen, entity labels, status icons, minimap, and a character panel. UI is drawn unlit on top of the world. In GPU (`ascii`/`graphics`) and capture modes the UI is rendered as a separate 2x2 pixel-per-cell pass; terminal mode renders UI at full character size.

**Chunk system**: `ChunkedGrid` is divided into 64x64 `Chunk`s. Each chunk tracks `active`, `modified`, `was_modified`, `generated`, and `dirty` (an optional bounding rect of cells that need processing). `save_chunk(path, cx, cy)` and `load_chunk(path, cx, cy)` serialize chunk cells via 12-byte binary format. Cell serialization is handled by `Cell::to_bytes()` / `Cell::from_bytes()`. `Chunk::generated` is set when a chunk is generated or loaded from cache, preventing accidental regeneration.

**Dirty rect optimization** (Noita-style): Each chunk maintains a `dirty: Option<(i32, i32, i32, i32)>` bounding rect of cells that need CA processing. When a cell changes via `set`, `set_material`, `cells_swap`, or `set_cell_index`, the chunk's dirty rect is expanded to include that cell ±1 (for neighbor influence). The CA step only iterates cells within dirty rects, skipping chunks with no dirty rect entirely. Liquids and gases (water, lava, acid, steam, fire, smoke) re-mark themselves dirty after processing so they continue to flow and react. Sand and other solids can sleep when at rest. `heat_transfer` only processes cells within dirty rects and reuses its temperature buffer across frames. `update_active_chunks` activates chunks with dirty rects in addition to chunks near entities. This reduces CA step time from ~500μs to ~25μs on a 250x250 grid.

**World cache**: main game seeds are saved in `Game::seed` and written to `cache/worlds/<seed>/`. Each cached world stores per-chunk binary files plus a `meta.json` with player spawn and item placement. `Game::init_world()` loads the cache if it exists; otherwise it generates the spawn region and saves it. This makes Noita-scale worlds load instantly after the first visit.

**Vertical biome progression**: World Y (`cy`) selects biome per chunk:
- `cy < 2` — surface (grass, dirt, stone, trees, pools, dunes)
- `2 <= cy < 6` — caves (stone, CA-carved empty space, lava/water/acid pools)
- `cy >= 6` — dungeon (BSP rooms, corridors, stone walls)

**Chunk streaming**: `Game::stream_chunks()` is called every `fixed_update`. It loads cached chunks (or generates new ones) in a 3-chunk radius around the player, saves modified chunks beyond that radius, and unloads distant chunks. Streaming is active for infinite grids and for bounded grids larger than 2048×2048; test/AI 250×250 grids skip streaming.

**Vertical descent**: `MaterialId::Stairs` is a solid feature material. Player stands on stairs and presses `>` to descend. `Game::descend()` increments depth, resets the world, and respawns the player at the top. HUD shows current depth.

## Key Conventions

- **No comments in code** unless explicitly requested.
- **Cell colors are inline** — renderers read `cell.fg`/`cell.bg` directly, never lookup `MaterialRegistry` in render path. Registry is for physics properties only (density, solid, flammable, etc.).
- **Vector movement** — `Player::move_left/right` sets velocity directly (`set_horizontal_vel`), no accumulation. `stop_horizontal` zeroes it. Jump is edge-triggered (press only, not held).
- **Slope stepping** — `update_rigid_entity` tries stepping up 1 cell before resolving X collision, enabling walking up slopes without jumping.
- **Adaptive viewport** — `check_resize()` in both Vulkan renderers recreates swapchain on window resize. `grid_w`/`grid_h` recalculated from extent / 16 (cell size = 16x16 px). Wayland uses `window.inner_size()` (surface extent is undefined).
- **`Cell::new()` copies colors from `MaterialRegistry`** at creation time. Per-cell color variation is possible by modifying `cell.fg`/`cell.bg` after creation.
- **Auto-constraints** — `BodyTemplate::auto_constraints(n)` connects all parts to all others (n^2). Works for any template shape. Simpler than manual constraint lists.
- **Item pickup** — `Game::update_item_pickup()` scans items within 1.5 cells of the player and adds them to `player.inventory`.
- **Stat-based health** — Entity max health derived from `base + toughness * 5 + level * 10`. `recalc_max_health()` called on `add_xp` level-up.
- **Status effects** — `update_status_effects()` applies damage for poison/bleeding/fire and cancels movement for frozen; effects expire when their timer reaches zero.
- **Random seeding** — `Game::new()` uses a fixed seed for tests/AI sessions; `Game::new_random()` seeds from system time and is used by terminal/ascii/graphics modes. Cached worlds are keyed by seed.
- **Dirty rects** — `ChunkedGrid::mark_dirty(x, y)` expands the chunk's dirty rect to include (x,y) ±1 and propagates to neighbor chunks at boundaries. `cells_swap` and `set_cell_index` call `mark_dirty` automatically. `set` and `set_material` also call `mark_dirty`. The CA step clears each chunk's dirty rect at the start of processing and rebuilds it from cell modifications during processing.
- **World scale**: `WORLD_SCALE = 5` in `worldgen.rs` — all world features (trees, pools, walls, dunes, rooms, corridors, terrain amplitude) are multiplied by this factor. Change this constant to adjust entity-to-world size ratio.

## Module Layout

```
src/
  main.rs              # CLI, GpuRenderer trait, run_gpu_mode<R>(), event loops
  lib.rs               # pub mod declarations
  game.rs              # Game struct, world gen, fixed_update, collision, combat, slime AI
  input.rs             # Terminal input (crossterm, InputHandler) — terminal mode only
  world/               # Cell, MaterialId, MaterialRegistry, ChunkedGrid, Grid (legacy), Chunk, CellularAutomaton, WorldGenerator, WorldCache
  physics/             # VerletSolver, SubBody (with color field), Constraint, resolve_grid_collision
  entity/              # Entity (rigid/ragdoll), EntityManager, Player, BodyTemplate, Item, ItemManager
  render/              # terminal.rs, vulkan.rs (ASCII), graphics.rs (cells), lighting.rs, window_input.rs
  ai/                  # GameSession, AiAction, pipe protocol, replay, scenarios
  ui/                  # UiLayer, HUD, messages, damage numbers, edge indicators
```

## PLAN.md

Detailed roadmap with 8 phases, architecture decisions, milestones, and cross-platform support table. Read it for design context before making architectural changes. Player combat is ranged (projectiles), not melee — this is a design decision recorded in Phase 1.
