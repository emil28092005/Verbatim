# AGENTS.md

## Build & Run

```sh
cargo build                          # debug build
cargo run --release -- --mode ascii       # Vulkan window, ASCII glyphs (default, recommended)
cargo run --release -- --mode graphics    # Vulkan window, colored cells, 16:9 window (recommended)
cargo run -- --mode ascii            # Vulkan window, ASCII glyphs (debug build, slower)
cargo run -- --mode graphics         # Vulkan window, colored cells, 16:9 window (debug build, slower)
cargo run -- --mode terminal         # ANSI terminal mode
cargo run -- --mode pipe             # JSON stdin/stdout for AI agents
cargo run -- --mode test             # run all JSON scenarios
cargo run -- --mode headless --headless-ticks 60  # dump to headless_dump.txt
cargo run -- --mode capture --headless-ticks 60    # render graphics-like PNG to capture.png
cargo run --release -- --mode benchmark --benchmark-ticks 600 --benchmark-renderer graphics  # FPS benchmark
```

Rust edition 2024, requires rustc >= 1.96. Vulkan 1.2+ required for `ascii`/`graphics` modes (falls back to terminal). Use `--release` for playable frame rates; GPU modes are CPU-bound in debug builds due to the cellular-automaton simulation. The GPU event loop is capped at 60 FPS.

## Tests

```sh
cargo test                           # all 171 integration tests
cargo test --test physics_sand       # single test file
cargo test --test slime              # slime-specific tests
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

**Source of truth**: `Grid` (250x250) of `Cell` structs. Each `Cell` stores `material`, `temp`, `fg`/`bg` color, `variant` inline. No double buffer. Grid is divided into 64x64 `Chunk`s with active flags and per-chunk persistence.

**Four entity kinds**: `Player`, `Goblin`, `Slime`, `Corpse`. Three physics types: cellular (CA materials in grid), rigid (alive entities, AABB + slope stepping), ragdoll (corpses, Verlet constraints).

**Three render modes**: `terminal` (crossterm ANSI), `ascii` (Vulkan glyph atlas + instanced), `graphics` (Vulkan colored quads). All three read the same `Grid` + `EntityManager` and an optional `LightGrid` overlay.

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

**Chunk system**: `Grid` is divided into 64x64 `Chunk`s. Each chunk tracks `active` and `modified`. `save_chunk(path, cx, cy)` and `load_chunk(path, cx, cy)` serialize chunk cells via 12-byte binary format. Cell serialization is handled by `Cell::to_bytes()` / `Cell::from_bytes()`.

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

## Module Layout

```
src/
  main.rs              # CLI, GpuRenderer trait, run_gpu_mode<R>(), event loops
  lib.rs               # pub mod declarations
  game.rs              # Game struct, world gen, fixed_update, collision, combat, slime AI
  input.rs             # Terminal input (crossterm, InputHandler) — terminal mode only
  world/               # Cell, MaterialId, MaterialRegistry, Grid, Chunk, CellularAutomaton
  physics/             # VerletSolver, SubBody (with color field), Constraint, resolve_grid_collision
  entity/              # Entity (rigid/ragdoll), EntityManager, Player, BodyTemplate, Item, ItemManager
  render/              # terminal.rs, vulkan.rs (ASCII), graphics.rs (cells), lighting.rs, window_input.rs
  ai/                  # GameSession, AiAction, pipe protocol, replay, scenarios
  ui/                  # UiLayer, HUD, messages, damage numbers, edge indicators
```

## PLAN.md

Detailed roadmap with 8 phases, architecture decisions, milestones, and cross-platform support table. Read it for design context before making architectural changes. Player combat is ranged (projectiles), not melee — this is a design decision recorded in Phase 1.
