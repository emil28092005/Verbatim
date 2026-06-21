# Verbatim — Multi-Layer World Plan

> Phase 6: separate world layers for temperature, gas/air, pressure, and light.
> Created June 2026. Replaces the inline `Cell.temp` with parallel per-chunk arrays.

## Current State

All world data is packed into a single `Cell` struct:

```rust
pub struct Cell {
    pub material: MaterialId,      // 1 byte
    pub temp: f32,                 // 4 bytes  <-- will be removed
    pub updated_this_tick: bool,   // 1 byte
    pub variant: u8,               // 1 byte
    pub fg: [u8; 3],              // 3 bytes
    pub bg: [u8; 3],              // 3 bytes
}                                 // ~13 bytes
```

**Problems:**
- Every `temp` modification copies the entire `Cell` through `grid.get()`/`grid.set()` + triggers `mark_dirty()`
- `heat_transfer()` already uses a temporary `Vec<f32>` buffer but writes results back into `Cell`
- No pressure field exists
- No gas composition field exists (`Empty` = air, `static_ = true`, no flow)
- Lighting is ephemeral viewport-sized `LightGrid`, not persisted in world space

## Target Architecture

Parallel arrays inside `Chunk`, shared dirty rects:

```
Chunk (64x64 = 4096 cells)
  cells:      Vec<Cell>         ~36 KB  (material + color, no temp)
  temps:      Vec<f32>          16 KB   (temperature)
  pressure:   Vec<u8>            4 KB   (0-255, 128 = atmospheric)
  gas_type:   Vec<u8>            4 KB   (0=air, 1=smoke, 2=poison, 3=CO2, 4=steam)
  gas_density:Vec<u8>            4 KB   (0-255 concentration)
  light:      Vec<[u8; 3]>      12 KB   (world-space RGB)
                              --------
                              ~76 KB per chunk
```

**Why inside Chunk (not separate ChunkedGrids):**
- Reuse dirty rects, active chunks, streaming, serialization
- One `chunk_at()` call instead of multiple
- Layers updated in a single pass over the dirty rect

## New Struct Definitions

### Cell (cell.rs) — temp removed

```rust
pub struct Cell {
    pub material: MaterialId,
    pub updated_this_tick: bool,
    pub variant: u8,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
}                                 // ~9 bytes
```

`to_bytes()` returns `[u8; 8]` (was `[u8; 12]`):
| Offset | Bytes | Field |
|--------|-------|-------|
| 0 | 1 | material |
| 1 | 1 | variant |
| 2-4 | 3 | fg |
| 5-7 | 3 | bg |

`updated_this_tick` is not serialized (transient).

### Chunk (chunk.rs) — 5 new arrays

```rust
pub struct Chunk {
    pub cells: Vec<Cell>,
    pub temps: Vec<f32>,
    pub pressure: Vec<u8>,
    pub gas_type: Vec<u8>,
    pub gas_density: Vec<u8>,
    pub light: Vec<[u8; 3]>,
    pub active: bool,
    pub modified: bool,
    pub was_modified: bool,
    pub generated: bool,
    pub dirty: Option<(i32, i32, i32, i32)>,
}
```

`Chunk::new()` initializes:
- `cells`: 4096 × `Cell::empty()`
- `temps`: 4096 × `20.0` (ambient temperature)
- `pressure`: 4096 × `128` (atmospheric)
- `gas_type`: 4096 × `0` (air)
- `gas_density`: 4096 × `0` (no gas)
- `light`: 4096 × `[0, 0, 0]` (no light, computed later)

### ChunkedGrid (chunked_grid.rs) — new accessor methods

```rust
pub fn get_temp(&self, x: i32, y: i32) -> f32
pub fn set_temp(&mut self, x: i32, y: i32, t: f32)
pub fn get_pressure(&self, x: i32, y: i32) -> u8
pub fn set_pressure(&mut self, x: i32, y: i32, p: u8)
pub fn get_gas(&self, x: i32, y: i32) -> (u8, u8)       // (type, density)
pub fn set_gas(&mut self, x: i32, y: i32, gas_type: u8, density: u8)
pub fn get_light(&self, x: i32, y: i32) -> [u8; 3]
pub fn set_light(&mut self, x: i32, y: i32, rgb: [u8; 3])
```

All `set_*` methods call `mark_dirty(x, y)` (shared dirty rect).

`get_temp` for out-of-bounds returns `20.0` (ambient).
`get_pressure` for out-of-bounds returns `128` (atmospheric).
`get_gas` for out-of-bounds returns `(0, 0)` (clean air).
`get_light` for out-of-bounds returns `[0, 0, 0]`.

## Simulation Steps

### New `fixed_update()` order

```
1. stream_chunks()
2. update_active_chunks()
3. ca.step()           -- material CA (sand, water, lava, etc.)
4. heat_transfer()     -- temperature diffusion on temps[] directly
5. gas_step()          -- gas flow CA (NEW)
6. pressure_step()     -- pressure equalization (NEW)
7. light_step()        -- world-space light update, every N ticks (NEW)
8. update_entities()
9. update_slime_ai()
10. update_goblin_ai()
11. update_combat()
12. update_projectiles()
13. apply_world_damage()
14. decompose_corpses()
15. update_status_effects()   -- now includes gas damage
16. update_score()
17. update_item_pickup()
18. grid.swap_modified_flags()
```

### Step 3: CA Rules Refactor (cellular.rs)

All `cell.temp` references become `grid.get_temp(x, y)`:

| Method | Current | New |
|--------|---------|-----|
| `update_water` | `cell.temp > 100.0` → Steam | `grid.get_temp(x,y) > 100.0` → Steam |
| `update_lava` | `cell.temp < 400.0` → Stone | `grid.get_temp(x,y) < 400.0` → Stone |
| `update_lava` | `lava.temp -= 50.0` | `grid.set_temp(x,y, grid.get_temp(x,y) - 50.0)` |
| `update_steam` | `cell.temp < 80.0` → Water | `grid.get_temp(x,y) < 80.0` → Water |
| `update_fire` | `new_n.temp = 400.0` | `grid.set_temp(nx,ny, 400.0)` |
| `update_fire` | `new.temp -= 15.0` | `grid.set_temp(x,y, grid.get_temp(x,y) - 15.0)` |
| `update_flesh` | `cell.temp > 200.0` → Fire | `grid.get_temp(x,y) > 200.0` → Fire |
| `update_grass` | `cell.temp > 250.0` → Fire | `grid.get_temp(x,y) > 250.0` → Fire |
| `update_dirt` | `cell.temp < 0.0` → Stone | `grid.get_temp(x,y) < 0.0` → Stone |

Also: when CA creates a new material (e.g., Water → Steam), set the temp layer:
- `grid.set_temp(x, y, 110.0)` for steam from boiling water
- `grid.set_temp(x, y, 50.0)` for water from condensed steam
- `grid.set_temp(x, y, 400.0)` for fire from ignited material

### Step 4: heat_transfer() Refactor (cellular.rs)

**Before** (current): reads `cell.temp` via `grid.get()`, writes via `grid.set()`, copies Cell each time.

**After**: direct array access on `chunk.temps[]`.

```
for each active chunk with dirty rect:
    snapshot temps[dirty_rect + 1 margin] into self.temps_buffer
    for each cell in dirty_rect:
        avg = average of 4 neighbors from self.temps_buffer
        chunk.temps[idx] += (avg - chunk.temps[idx]) * conductivity * 0.1
        if phase transition threshold crossed:
            grid.set_material(x, y, new_material)  -- triggers mark_dirty
            grid.set_temp(x, y, new_temp)
```

**No `grid.get()`/`grid.set()` for temperature.** No Cell copying. No mark_dirty per pixel (only for phase transitions).

Performance: heat_transfer goes from ~500 get+set calls to ~500 direct array writes.

### Step 5: gas_step() — NEW (cellular.rs)

Gas CA rules per cell in dirty rect:

**Flow rules:**
- Gas rises if `gas_density > 0` and cell above is empty/gas with lower density
- Gas spreads horizontally to equalize density
- Gas cannot pass through solid cells (is_solid)
- Gas accumulates at ceilings (density increases upward)

**Gas types:**

| Type | ID | Behavior |
|------|----|----------|
| Air | 0 | Default, no effect |
| Smoke | 1 | Rises, blocks light slightly, fades over time |
| Poison | 2 | Rises, damages entities in contact, fades slowly |
| CO2 | 3 | Rises, suffocates fire (fire dies if CO2 density > threshold) |
| Steam | 4 | Rises, condenses to water at temp < 80, transparent |

**Material interactions:**
- `Fire` + `Air` → produces `CO2` + `Smoke`, consumes air density
- `Fire` + `CO2` (high density) → fire extinguishes
- `Lava` + `Water` → produces `Steam` (gas_type=4)
- `Acid` + `Flesh/Wood` → produces `Poison` gas
- `Steam` + `temp < 80` → condenses to `Water` (gas cleared)

**Gas update order:**
1. Material-to-gas transitions (fire produces CO2, lava+water → steam)
2. Gas flow (rise + spread)
3. Gas-to-material transitions (steam condenses, fire suffocates)
4. Gas fading (smoke/poison slowly dissipate)

### Step 6: pressure_step() — NEW (cellular.rs)

Pressure equalization for connected liquid/gas regions.

**Rules:**
- `128` = atmospheric pressure (default for empty cells)
- Liquids generate pressure by depth: `pressure = 128 + depth_from_surface * k`
- Pressure diffuses to neighbors: `p += (neighbor_p - p) * diffusion_rate`
- Solids block pressure transfer
- High pressure ( > 200) pushes liquids/gas through gaps
- Explosions: temporarily set pressure to 255, then diffuse

**Algorithm:**
```
for each active chunk with dirty rect:
    for each cell in dirty_rect:
        if cell is liquid or gas or empty:
            avg_p = average of 4 non-solid neighbors' pressure
            new_p = lerp(current_p, avg_p, 0.1)
            chunk.pressure[idx] = new_p
```

This enables:
- U-bend pipes (water level equalizes through connected path)
- Fountains (high pressure pushes water up)
- Gas displacement (fire creates pressure, pushes gas out)
- Explosions (pressure wave propagates, destroys weak materials)

### Step 7: light_step() — NEW (cellular.rs)

World-space persistent lighting, updated every N ticks (default N=10).

**Algorithm:**
```
every N ticks:
    for each active chunk:
        clear chunk.light to [0,0,0]
    for each light source in active chunks (Lava, Fire):
        ray-cast outward up to radius
        for each cell within radius with line-of-sight:
            attenuation = (1 - dist/radius)^2
            chunk.light[idx] += source.color * source.intensity * attenuation
            clamp to 255
```

**Consumers:**
- AI spectrum `render_light()` reads `grid.get_light()` instead of ephemeral `LightGrid`
- Terminal renderer: optional world-space light mode
- GPU renderers: stay as-is (compute lighting in shader, real-time)
- Capture renderer: can use world-space light for consistency

**Performance:**
- Only updates every 10 ticks (6 times per second at 60 FPS)
- Only processes active chunks with dirty rects
- Light sources gathered from `material_light()` (same as current)

### Step 8: Gas Damage in update_status_effects() (game.rs)

New status effect from gas:

```rust
fn update_status_effects(&mut self) {
    for e in self.entities.all_mut() {
        if e.alive {
            let (ex, ey) = e.center();
            let (gas_type, gas_density) = self.grid.get_gas(ex as i32, ey as i32);
            if gas_type == 2 && gas_density > 50 {
                // Poison gas: damage proportional to density
                e.health -= (gas_density as f32 - 50.0) * 0.1;
                e.status_effects.push(StatusEffect::Poisoned { timer: 60 });
            }
            if gas_type == 3 && gas_density > 100 {
                // CO2: suffocation damage
                e.health -= 2.0;
            }
            e.apply_status_effects();
        }
    }
}
```

## Serialization

### Multi-section chunk file format

```
File: cache/worlds/seed_<N>/chunk_<cx>_<cy>.bin

[4 bytes:  magic "VWM1"]           // Verbatim World Map v1
[1 byte:   version = 1]
[1 byte:   flags = 0]
[4 bytes:  cell_section_len]       // 8 * 4096 = 32768
[cell_section: 8 bytes x 4096]     // material(1) + variant(1) + fg(3) + bg(3)
[4 bytes:  temp_section_len]       // 4 * 4096 = 16384
[temp_section: 4 bytes x 4096]     // f32 little-endian
[2 bytes:  gas_section_len]        // 2 * 4096 = 8192
[gas_section: 2 bytes x 4096]      // type(1) + density(1)
[2 bytes:  pressure_section_len]   // 1 * 4096 = 4096
[pressure_section: 1 byte x 4096]  // u8
[4 bytes:  light_section_len]      // 3 * 4096 = 12288
[light_section: 3 bytes x 4096]    // R + G + B
```

Total: ~8 + 32768 + 16384 + 8192 + 4096 + 12288 = ~74 KB per chunk (was ~49 KB).

**Old cache is incompatible.** Run `rm -rf cache/worlds` after implementation.

### Backward compatibility

None. Old `chunk_*.bin` files (12 bytes/cell format) will fail to load with a clear error. The `meta.json` format stays the same (player position, items, depth).

## Renderer Changes

### GPU Grid Buffers (graphics.rs, vulkan.rs)

Current: one storage buffer with `material as u32` per cell.

New: additional storage buffers:
- `temp_buffer`: `f32` per viewport cell (for heat shimmer, temperature visualization)
- `gas_buffer`: `u32` per viewport cell (packed: `gas_type << 16 | gas_density`)
- `pressure_buffer`: `u32` per viewport cell (for pressure visualization)

Shaders (graphics.vert, cell.vert):
- Binding 0: material grid (existing, for `is_solid()` in line_of_sight)
- Binding 1: light sources (existing)
- Binding 2: temperature grid (NEW, optional use for heat shimmer)
- Binding 3: gas grid (NEW, optional use for fog/smoke overlay)

**Phase 1 implementation**: just upload the data, don't change shader visuals yet.
**Phase 2**: add heat shimmer (vertex displacement based on temp), gas fog (alpha overlay).

### Terminal Renderer (terminal.rs)

- `render()` can optionally use `grid.get_light()` (world-space) instead of CPU `compute_lighting()`
- Toggle with a flag or mode: `--mode terminal --world-light` (experimental)
- Default stays: ephemeral `compute_lighting()` for immediate accuracy

### Capture Renderer (capture.rs)

- Use `grid.get_light()` if available (world-space light)
- Fallback to `compute_lighting()` if light layer is all zeros

### AI Spectrum (spectrum.rs)

| Spectrum | Current source | New source |
|----------|---------------|------------|
| materials | `cell.material` | `cell.material` (unchanged) |
| temperature | `cell.temp` | `grid.get_temp(x, y)` |
| light | `LightGrid` (ephemeral) | `grid.get_light(x, y)` |
| entities | `cell.is_empty()` | `cell.is_empty()` (unchanged) |
| density | `MaterialRegistry` | `MaterialRegistry` (unchanged) |
| velocity | `cell.updated_this_tick` | `cell.updated_this_tick` (unchanged) |
| gas (NEW) | — | `grid.get_gas(x, y)` → type + density chars |
| pressure (NEW) | — | `grid.get_pressure(x, y)` → 0-9 scale char |

New spectrum commands in pipe protocol:
```json
{"cmd":"get_spectrum","spectrum":"gas","w":80,"h":25}
{"cmd":"get_spectrum","spectrum":"pressure","w":80,"h":25}
```

## Material Properties (material.rs)

New fields in `Material`:

```rust
pub struct Material {
    // ... existing fields ...
    pub gas_emission: (u8, u8),    // (gas_type, amount_per_tick) — 0 = none
    pub pressure_gen: u8,          // pressure added per tick (for explosions/lava)
}
```

| Material | gas_emission | pressure_gen |
|----------|-------------|--------------|
| Empty | (0, 0) | 0 |
| Sand | (0, 0) | 0 |
| Water | (0, 0) | 0 |
| Stone | (0, 0) | 0 |
| Lava | (0, 0) | 1 |
| Wood | (0, 0) | 0 |
| Flesh | (0, 0) | 0 |
| Bone | (0, 0) | 0 |
| Steam | (4, 0) | 0 |
| Fire | (3, 5) | 2 |
| Acid | (0, 0) | 0 |
| Smoke | (1, 0) | 0 |
| Grass | (0, 0) | 0 |
| Dirt | (0, 0) | 0 |
| Stairs | (0, 0) | 0 |

`Fire` emits CO2 (type 3) at 5 density/tick and generates 2 pressure/tick.
`Lava` generates 1 pressure/tick (heat expansion).

## Test Plan

### Updated tests (cell.temp → grid.get_temp)

All tests that access `cell.temp` via `GameSession` need updating:
- `tests/physics_lava.rs` — lava temperature checks
- `tests/physics_interactions.rs` — lava+water=steam, fire spread
- `tests/physics_water.rs` — water boiling
- `tests/integration.rs` — any temp-related assertions
- `tests/physics_acid.rs` — acid + organic → poison gas

### New tests

**Temperature layer:**
- `heat_transfer_diffuses_through_solid_material` — stone wall between hot/cold
- `heat_transfer_uses_conductivity` — wood conducts less than stone
- `lava_heats_adjacent_water_to_steam` — phase transition via heat diffusion
- `temperature_persists_across_chunk_boundary` — cross-chunk heat flow

**Gas layer:**
- `gas_rises_and_accumulates_at_ceiling` — smoke fills upward
- `fire_consumes_air_and_produces_co2` — fire reduces air, increases CO2
- `fire_extinguishes_in_high_co2` — fire dies without air
- `poison_gas_damages_entity` — entity takes damage in poison gas
- `steam_condenses_to_water_when_cold` — gas-to-material transition
- `lava_plus_water_produces_steam_gas` — material interaction creates gas
- `gas_does_not_pass_through_solid_walls` — containment test

**Pressure layer:**
- `pressure_equalizes_in_connected_liquids` — U-bend test
- `high_pressure_pushes_liquid_through_gap` — fountain test
- `explosion_creates_pressure_wave` — pressure propagation
- `solid_blocks_pressure_transfer` — isolation test

**Light layer:**
- `light_persists_in_world_space` — light stored in chunk
- `light_blocked_by_solid_walls` — shadow casting
- `light_updates_when_source_removed` — fire extinguished → darkness
- `lava_emits_light_in_world_space` — light source test

**Serialization:**
- `chunk_save_load_roundtrip_preserves_all_layers` — temp, gas, pressure, light
- `old_cache_format_rejected` — error on loading v0 format

## Implementation Order

| Step | Files | Description |
|------|-------|-------------|
| 1 | `cell.rs` | Remove `temp` from `Cell`, update `to_bytes`/`from_bytes` to 8 bytes |
| 2 | `chunk.rs` | Add `temps`, `pressure`, `gas_type`, `gas_density`, `light` arrays, init in `new()` |
| 3 | `chunked_grid.rs` | Add `get_temp`/`set_temp`/`get_pressure`/`set_pressure`/`get_gas`/`set_gas`/`get_light`/`set_light` methods |
| 4 | `chunked_grid.rs` | Update `save_chunk`/`load_chunk` to multi-section format |
| 5 | `cellular.rs` | Refactor `heat_transfer()` to direct array access on `temps[]` |
| 6 | `cellular.rs` | Refactor all CA rules: `cell.temp` → `grid.get_temp()` / `grid.set_temp()` |
| 7 | `cellular.rs` | Implement `gas_step()` — gas flow + material interactions |
| 8 | `cellular.rs` | Implement `pressure_step()` — pressure equalization |
| 9 | `cellular.rs` | Implement `light_step()` — world-space lighting (every N ticks) |
| 10 | `game.rs` | Update `fixed_update()`: add gas_step, pressure_step, light_step calls |
| 11 | `game.rs` | Update `update_status_effects()`: gas damage (poison, CO2) |
| 12 | `material.rs` | Add `gas_emission`, `pressure_gen` fields to `Material` |
| 13 | `render/graphics.rs` | Upload temp/gas/pressure buffers to GPU |
| 14 | `render/vulkan.rs` | Same as graphics.rs |
| 15 | `render/lighting.rs` | Update `compute_lighting` to optionally read world-space light |
| 16 | `render/terminal.rs` | Optional world-space light mode |
| 17 | `render/capture.rs` | Use world-space light if available |
| 18 | `ai/spectrum.rs` | Update temperature/light spectrums, add gas + pressure spectrums |
| 19 | `ai/protocol.rs` | Add gas/pressure spectrum commands |
| 20 | `ai/state.rs` | Update `CellInfo` to include gas/pressure data |
| 21 | `ai/session.rs` | Update `find_material` for infinite mode (separate fix) |
| 22 | `tests/*.rs` | Update all temp-related tests, add new layer tests |
| 23 | — | `rm -rf cache/worlds` (old cache incompatible) |
| 24 | — | `cargo test` — all green |
| 25 | — | `cargo run --release -- --mode benchmark` — verify no regression |

## File Impact Map

| File | Changes |
|------|---------|
| `src/world/cell.rs` | Remove `temp` field, update `to_bytes`/`from_bytes` (12→8 bytes) |
| `src/world/chunk.rs` | 5 new arrays, init, accessor methods, `reset_tick_flags` unchanged |
| `src/world/chunked_grid.rs` | 8 new accessor methods, multi-section serialization, `ensure_chunk` init new arrays |
| `src/world/cellular.rs` | `heat_transfer` refactor, `gas_step`, `pressure_step`, `light_step`, CA rule temp changes |
| `src/world/material.rs` | `gas_emission`, `pressure_gen` fields on `Material` |
| `src/game.rs` | `fixed_update` new steps, gas damage in `update_status_effects` |
| `src/render/graphics.rs` | 3 new GPU buffers (temp, gas, pressure), upload code |
| `src/render/vulkan.rs` | Same as graphics.rs |
| `src/render/lighting.rs` | Optional world-space light mode in `compute_lighting` |
| `src/render/terminal.rs` | Optional `--world-light` flag |
| `src/render/capture.rs` | World-space light fallback |
| `src/ai/spectrum.rs` | Temp/light from layers, new gas/pressure spectrums |
| `src/ai/protocol.rs` | New spectrum commands |
| `src/ai/state.rs` | `CellInfo` + gas/pressure fields |
| `tests/*.rs` | Update ~10 test files, add ~20 new tests |

## Performance Expectations

| Metric | Before | Expected After |
|--------|--------|---------------|
| Cell copy per get() | 13 bytes | 9 bytes (-31%) |
| heat_transfer per cell | 2 Cell copies + mark_dirty | 1 array write (no copy, no dirty) |
| Chunk memory | ~53 KB | ~76 KB (+43%) |
| Active chunk memory (9 chunks) | ~477 KB | ~684 KB |
| CA step time | ~0.5 ms | ~0.4 ms (fewer copies) |
| heat_transfer time | ~0.3 ms | ~0.1 ms (direct array) |
| gas_step time | N/A | ~0.2 ms (new) |
| pressure_step time | N/A | ~0.1 ms (new) |
| light_step time (every 10 ticks) | N/A | ~0.5 ms (amortized ~0.05 ms/tick) |
| Total fixed_update | ~1.5 ms | ~1.3 ms |
| Render frame | ~4 ms | ~4.5 ms (+3 buffers) |

## Cross-Layer Interactions

```
Fire (material) 
  → heats temp layer (heat_transfer)
  → emits CO2 + Smoke (gas layer)
  → generates pressure (pressure layer)
  → emits light (light layer)

Lava (material)
  → heats temp layer
  → generates pressure
  → emits light
  → + Water → Steam (gas layer) + temp drop

Temperature layer
  → Water > 100°C → Steam (material + gas)
  → Lava < 400°C → Stone (material)
  → Steam < 80°C → Water (material, gas cleared)
  → Wood > 300°C → Fire (material)

Gas layer
  → CO2 high → Fire extinguishes (material)
  → Poison → entity damage (status effects)
  → Steam + cold temp → Water (material)

Pressure layer
  → High pressure → pushes liquid through gaps
  → Explosion → pressure wave → terrain destruction
  → Lava → constant pressure generation
```

## Open Questions

| Question | Default | Notes |
|----------|---------|-------|
| Light update frequency | Every 10 ticks | Configurable, tradeoff: accuracy vs perf |
| Gas diffusion rate | 0.1 | How fast gas spreads per tick |
| Pressure diffusion rate | 0.1 | How fast pressure equalizes |
| Gas fade rate | 1 per 60 ticks | Smoke/poison slowly dissipate |
| Max gas density | 255 | u8 limit, may need f32 if finer |
| Pressure as u8 or f32? | u8 (0-255) | Simpler, 128=atmospheric. f32 if precision needed |
| Steam as gas or material? | Gas (type 4) | Current: Steam is a material. New: Steam is a gas that condenses to Water |
| CO2 threshold for fire death | density > 150 | Fire extinguishes when CO2 concentration is high |
| Poison damage threshold | density > 50 | Entity takes damage above this concentration |
