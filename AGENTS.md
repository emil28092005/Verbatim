# AGENTS.md

## Build & Run

```sh
cargo build                          # debug build
cargo run -- --mode ascii            # Vulkan window, ASCII glyphs (default)
cargo run -- --mode graphics         # Vulkan window, colored cells
cargo run -- --mode terminal         # ANSI terminal mode
cargo run -- --mode pipe             # JSON stdin/stdout for AI agents
cargo run -- --mode test             # run all JSON scenarios
cargo run -- --mode headless --headless-ticks 60  # dump to headless_dump.txt
```

Rust edition 2024, requires rustc >= 1.96. Vulkan 1.2+ required for `ascii`/`graphics` modes (falls back to terminal).

## Tests

```sh
cargo test                           # all 122 integration tests
cargo test --test physics_sand       # single test file
cargo test --test slime              # slime-specific tests
cargo run -- --mode test --scenario-dir scenarios  # 14 JSON scenarios
```

Tests are in `tests/*.rs`, use `verbatim::ai::GameSession` API (not the render loop). Scenarios are in `scenarios/*.json`.

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

**Source of truth**: `Grid` (250x250) of `Cell` structs. Each `Cell` stores `material`, `temp`, `fg`/`bg` color, `variant` inline. No double buffer.

**Four entity kinds**: `Player`, `Goblin`, `Slime`, `Corpse`. Three physics types: cellular (CA materials in grid), rigid (alive entities, AABB + slope stepping), ragdoll (corpses, Verlet constraints).

**Three render modes**: `terminal` (crossterm ANSI), `ascii` (Vulkan glyph atlas + instanced), `graphics` (Vulkan colored quads). All three read the same `Grid` + `EntityManager`.

**GpuRenderer trait** (`main.rs`): unifies `VulkanRenderer` and `GraphicsRenderer` behind `run_gpu_mode<R>()`. Both have identical event loops; only shader/instance format differs.

**Game loop**: `Game::fixed_update()` = CA step -> rigid update -> ragdoll update -> slime AI -> combat -> damage. Called from event loop with fixed 16ms accumulator. `Game::run()` is terminal-only (uses `InputHandler` with crossterm). GPU modes use `run_gpu_mode` with `WindowInput` (winit PhysicalKey, layout-agnostic).

**BodyTemplate** (`entity/body_template.rs`): data-driven entity body definitions. JSON-serializable. `build_humanoid()` delegates to `template_for_kind().apply_to()`. To add a new creature shape, add a `BodyTemplate` constructor + match arm in `template_for_kind`. Each `SubBody` has a `color: [u8; 4]` field for per-part coloring.

**Combat**: `update_combat()` checks AABB overlap between player and all alive enemies. Goblin = 8 dmg, Slime = 5 dmg per 20 ticks on contact. Knockback applied to player. Player combat style is ranged (projectiles planned, not yet implemented).

**Slime AI**: `update_slime_ai()` makes slimes jump toward player every 60 ticks when within 40 cells. Jump power scales with proximity.

## Key Conventions

- **No comments in code** unless explicitly requested.
- **Cell colors are inline** — renderers read `cell.fg`/`cell.bg` directly, never lookup `MaterialRegistry` in render path. Registry is for physics properties only (density, solid, flammable, etc.).
- **Vector movement** — `Player::move_left/right` sets velocity directly (`set_horizontal_vel`), no accumulation. `stop_horizontal` zeroes it. Jump is edge-triggered (press only, not held).
- **Slope stepping** — `update_rigid_entity` tries stepping up 1 cell before resolving X collision, enabling walking up slopes without jumping.
- **Adaptive viewport** — `check_resize()` in both Vulkan renderers recreates swapchain on window resize. `grid_w`/`grid_h` recalculated from extent / 10 (cell size = 10x10 px). Wayland uses `window.inner_size()` (surface extent is undefined).
- **`Cell::new()` copies colors from `MaterialRegistry`** at creation time. Per-cell color variation is possible by modifying `cell.fg`/`cell.bg` after creation.
- **Auto-constraints** — `BodyTemplate::auto_constraints(n)` connects all parts to all others (n^2). Works for any template shape. Simpler than manual constraint lists.

## Module Layout

```
src/
  main.rs              # CLI, GpuRenderer trait, run_gpu_mode<R>(), event loops
  lib.rs               # pub mod declarations
  game.rs              # Game struct, world gen, fixed_update, collision, combat, slime AI
  input.rs             # Terminal input (crossterm, InputHandler) — terminal mode only
  world/               # Cell, MaterialId, MaterialRegistry, Grid, CellularAutomaton
  physics/             # VerletSolver, SubBody (with color field), Constraint, resolve_grid_collision
  entity/              # Entity (rigid/ragdoll), EntityManager, Player, BodyTemplate
  render/              # terminal.rs, vulkan.rs (ASCII), graphics.rs (cells), window_input.rs
  ai/                  # GameSession, AiAction, pipe protocol, replay, scenarios
```

## PLAN.md

Detailed roadmap with 8 phases, architecture decisions, milestones, and cross-platform support table. Read it for design context before making architectural changes. Player combat is ranged (projectiles), not melee — this is a design decision recorded in Phase 1.
