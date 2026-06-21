# Verbatim — GPU Optimization Plan

> Autonomously drive the game to full GPU optimization.
> Created June 2026. All work is self-contained — no user interaction needed.

## Final Results (600-tick benchmark)

| Mode | Baseline FPS | Final FPS | Improvement | Baseline Render | Final Render | CA Step |
|------|-------------|-----------|-------------|-----------------|--------------|---------|
| Graphics | 386 | **531** | +38% | 1699us | **1013us** (-40%) | 347us |
| ASCII | 313 | **402** | +28% | 2346us | **1502us** (-36%) | 382us |

Both modes far exceed the 60 FPS target. All 171 tests + 14 scenarios pass.

## Optimizations Completed

### Phase A: Benchmark Infrastructure ✓
- Added `--mode benchmark` with `--benchmark-ticks`, `--benchmark-renderer`, `--benchmark-output` CLI args
- Measures CA step, render, and total frame times with percentile stats
- Outputs JSON results file

### Phase B: Eliminate Wasted CPU Work ✓
- Skipped `lighting::compute_lighting()` for GPU modes in `main.rs`
- Pass `None` for lighting to GPU renderers

### Phase C: GPU Lighting Shader Optimization ✓
- Replaced naive O(N×R²) grid scan with O(N×S) light source list iteration
- CPU gathers light sources into compact buffer (max 64 sources, 32 bytes each)
- Uploaded via second storage buffer (binding 2 in ascii, binding 1 in graphics)
- `gather_sources_in_range()` only scans viewport + 30-cell margin
- `light_count` passed via push constants

### Phase D: Viewport-Aware CA ✓
- CA step iterates only active chunks instead of all 250×250 cells
- `apply_cell_rule()` helper avoids code duplication
- Heat transfer also iterates only active chunks

### Phase E: Instance Building Optimization ✓
- Replaced HashMap entity_map/item_map/shadow_map with flat viewport-sized arrays
- Direct array indexing instead of hashing — 42% render speedup in graphics mode
- Replaced HashMap atlas_map with flat 128-entry ASCII array in ascii renderer

### Phase G: Partial Grid Upload ✓
- Only upload viewport + 30-cell margin region to GPU (260×172 vs 250×250)
- Pre-allocated viewport arrays in renderer struct to avoid per-frame allocation

## Starting State

| System | Status | Notes |
|--------|--------|-------|
| World cells | 8×8 px | Reduced from 16×16 |
| UI cells | 2×2 px | UI_SCALE = 4 |
| CA simulation | CPU, full 250×250 | Active-chunk system exists but still iterates all cells |
| Lighting | GPU (vertex shader) | Both ascii + graphics renderers; naive O(N×R²) per cell |
| CPU lighting | Still computed in main.rs | Wasted work for GPU modes — must be skipped |
| Instance building | CPU, per-frame | Full viewport iteration, HashMaps for entity/item overlap |
| FPS | Unknown | Need benchmark tool to measure |
| Tests | 171 pass, 14 scenarios | Must stay green throughout |

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Frame time (ascii mode) | < 16ms (60 FPS) | Unknown |
| Frame time (graphics mode) | < 16ms (60 FPS) | Unknown |
| CA step (250×250) | < 0.5ms | ~0.5ms (active chunks) |
| Instance build | < 2ms | Unknown |
| GPU lighting | < 2ms | Unknown (naive shader) |
| Grid upload | < 0.5ms | Unknown |

## Phases

### Phase A: Benchmark Infrastructure

**Goal: automated FPS measurement without human interaction**

- [ ] Add `--mode benchmark` CLI mode
  - Runs game for N ticks (default 600 = 10 seconds at 60 FPS)
  - Uses Vulkan renderer (ascii or graphics, configurable via `--benchmark-mode ascii|graphics`)
  - No window input needed — auto-runs, collects frame times
  - Outputs: min/avg/p99 FPS, frame time distribution, per-subsystem timing
  - Writes results to `benchmark_results.json` and prints summary to stdout
  - Subsystem timing: CA step, instance build, grid upload, render, total
  - Uses `Instant::now()` around each subsystem in the game loop
- [ ] Add `tools/benchmark.py` — parses JSON results, compares runs, generates trend table

### Phase B: Eliminate Wasted CPU Work

**Goal: remove CPU work that GPU now handles**

- [ ] Skip `lighting::compute_lighting()` in `main.rs` for GPU modes
  - Add `uses_cpu_lighting()` to `GpuRenderer` trait (default true)
  - Vulkan + Graphics override to false
  - `run_gpu_mode` only computes CPU lighting if renderer needs it
- [ ] Remove `lighting` parameter from GPU renderers' render signatures if unused
  - Keep trait signature compatible (pass None for GPU)
- [ ] Skip CPU `apply_light_rgba` in instance building for GPU renderers (already done)

### Phase C: Optimize GPU Lighting Shader

**Goal: reduce per-vertex lighting cost from O(R²) to O(S) where S = light source count**

Current shader scans a 60×60 area per cell looking for light sources. Most cells have zero nearby sources.

- [ ] CPU-side: gather light sources each frame into a compact buffer (max 64 sources)
  - Each source: x, y, radius, color (16 bytes)
  - Upload via a second storage buffer or uniform buffer
- [ ] Shader: iterate over light sources list instead of scanning grid
  - For each source: check distance < radius, then line_of_sight
  - O(S) per cell instead of O(R²)
  - S is typically 5-20 (lava pools, fires)
- [ ] Keep grid storage buffer for `is_solid()` checks in line_of_sight
- [ ] Benchmark before/after

### Phase D: Optimize CA — Viewport-Aware Simulation

**Goal: only simulate cells that matter**

Current: `update_active_chunks` activates chunks near entities/items/modified. But the CA step still iterates all 250×250 cells checking chunk active flags.

- [ ] Build a compact list of active chunk ranges at the start of each tick
  - `active_chunks: Vec<(cx, cy)>` — only iterate these
- [ ] CA step iterates only active chunks, not all 250×250
  - For each active chunk: iterate its 64×64 cells
  - Skip inactive chunks entirely (no bounds check per cell)
- [ ] Add a margin around the viewport: always simulate visible chunks + 1 chunk border
  - Ensures materials flowing into view are simulated
- [ ] Benchmark before/after

### Phase E: Optimize Instance Building

**Goal: reduce per-frame CPU overhead for preparing render data**

Current: iterates all viewport cells, uses HashMaps for entity/item overlap.

- [ ] Replace HashMap entity_map with a 2D array (viewport-sized)
  - `[[u32; VW]; VH]` — entity priority + index packed into u32
  - Avoids hashing per cell
- [ ] Same for item_map: `[[Option<[u8;4]>; VW]; VH]`
- [ ] Skip background_color hash for empty cells — precompute star pattern
  - Stars are deterministic by world position; cache the hash pattern
- [ ] Consider dirty-cell tracking: only update changed instances
  - Keep previous frame's instance buffer; diff against new state
  - Only write changed ColorInstance/CellInstance entries
  - Needs tracking of which cells changed (chunk modified flags can help)
- [ ] Benchmark before/after

### Phase F: GPU Compute Shader for CA (if needed)

**Goal: move cellular automaton to GPU compute**

Only if Phase D doesn't bring CA step below 0.5ms.

- [ ] Create compute shader `ca.comp` — one workgroup per chunk (64×64)
  - Read grid from storage buffer
  - Apply CA rules per cell
  - Write back to storage buffer
  - Use shared memory for chunk border exchange
- [ ] Double-buffer: ping-pong between two grid buffers
- [ ] CPU reads back only active chunks for entity physics
- [ ] Fallback: keep CPU CA for terminal/headless/test modes
- [ ] Benchmark before/after

### Phase G: Grid Upload Optimization

**Goal: minimize data transferred CPU→GPU per frame**

Current: full 250×250 grid (250KB) uploaded every frame for lighting.

- [ ] Only upload changed chunks
  - Use chunk `modified` flags to build a list of changed regions
  - Upload only changed regions via `vkCmdUpdateBuffer` or per-chunk sub-range writes
- [ ] Alternatively: use a staging buffer and `vkCmdCopyBuffer` for only dirty regions
- [ ] Consider keeping grid entirely on GPU if Phase F is implemented
  - CA runs on GPU, entity physics reads back only entity-adjacent cells
- [ ] Benchmark before/after

### Phase H: Final Verification

- [ ] Run full test suite: `cargo test` + scenarios
- [ ] Run benchmark in both ascii and graphics modes
- [ ] Compare FPS before/after all optimizations
- [ ] Document results in `benchmark_results.json` and summary in this file
- [ ] Update AGENTS.md with any new conventions

## Benchmark Protocol

```
# Baseline (before any optimization)
cargo run --release -- --mode benchmark --benchmark-ticks 600 --benchmark-mode ascii
cargo run --release -- --mode benchmark --benchmark-ticks 600 --benchmark-mode graphics

# After each phase
cargo run --release -- --mode benchmark --benchmark-ticks 600 --benchmark-mode ascii
cargo run --release -- --mode benchmark --benchmark-ticks 600 --benchmark-mode graphics
```

Each benchmark run produces:
```json
{
  "mode": "ascii",
  "ticks": 600,
  "total_time_ms": 10023.4,
  "avg_fps": 59.8,
  "min_fps": 52.1,
  "p99_fps": 57.3,
  "avg_frame_time_ms": 16.72,
  "p99_frame_time_ms": 19.2,
  "subsystems": {
    "ca_step_avg_ms": 0.48,
    "instance_build_avg_ms": 2.1,
    "grid_upload_avg_ms": 0.3,
    "render_avg_ms": 8.2,
    "lighting_avg_ms": 0.0
  }
}
```

## Key Constraints

- All 171 tests + 14 scenarios must pass after every phase
- Terminal/headless/test/pipe modes must continue working (CPU path intact)
- No visual regression — but since we can't visually inspect, rely on:
  - ASCII capture output (`--mode capture`) for pixel comparison
  - Test suite correctness
  - Frame timing stability
- Shaders must compile cleanly via `glslangValidator`
- No new external dependencies unless absolutely necessary

## File Impact Map

| File | Phases | Changes |
|------|--------|---------|
| `src/main.rs` | A, B | Benchmark mode, skip CPU lighting |
| `src/game.rs` | A, D, E | Timing instrumentation, viewport CA, instance arrays |
| `src/world/cellular.rs` | D, F | Active-chunk iteration, compute shader |
| `src/world/grid.rs` | D, G | Active chunk list, dirty region tracking |
| `src/render/vulkan.rs` | C, E, G | Light source buffer, instance optimization, partial upload |
| `src/render/graphics.rs` | C, E, G | Same as vulkan.rs |
| `src/render/mod.rs` | A, B | Renderer trait changes |
| `assets/shaders/cell.vert` | C | Light source list iteration |
| `assets/shaders/cell.frag` | C | (no change expected) |
| `assets/shaders/graphics.vert` | C | Light source list iteration |
| `assets/shaders/graphics.frag` | C | (no change expected) |
| `assets/shaders/ca.comp` | F | New compute shader |
| `tools/benchmark.py` | A | New tool |

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| Jun 21 | Start with benchmark tool | Can't optimize what we can't measure |
| Jun 21 | Light source buffer before CA compute | Bigger win for less effort |
| Jun 21 | Keep CPU CA as fallback | Terminal/test modes need it |
