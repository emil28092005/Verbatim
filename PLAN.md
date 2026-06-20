# Verbatim — Development Plan

> ASCII physics RPG. Noita's cellular automaton + Caves of Qud's RPG depth.
> Every symbol is a material with physics. Every entity is a body with mass.

## Current State (June 2026)

### What Works

| System | Status | Details |
|--------|--------|---------|
| Cellular automaton | Working | 14 materials: sand, water, stone, lava, wood, flesh, bone, steam, fire, acid, smoke, grass, dirt, empty |
| Rigid entities | Working | AABB collider, sliding on surfaces, 27 sub-bodies (5x5 + arm), player + goblins |
| Ragdoll corpses | Working | Verlet constraints, death = rigid→ragdoll transition with inherited velocity |
| Terminal renderer | Working | Full terminal size, ANSI truecolor, diff-based rendering |
| AI pipe protocol | Working | JSON stdin/stdout, 16 commands, full state export |
| Test framework | Working | 28 Rust tests + 8 JSON scenarios, all passing |
| Replay system | Working | Seeded determinism, record/playback, play_until_tick |
| World generation | Basic | Sinusoidal terrain, water/lava/acid pools, wood structure, sand dune, stone wall |

### Architecture

```
Source of truth: text grid (250x250, char + material + temp)

Three entity types:
  1. Cellular  — materials in grid, per-cell CA rules
  2. Rigid     — alive entities, AABB collider, single velocity
  3. Ragdoll   — corpses, loose Verlet bodies, independent physics

Game loop: fixed 60Hz timestep
  Physics tick: CA step → rigid update → ragdoll update → damage
  Render: terminal (ANSI) or pipe (JSON) or headless (file dump)
```

### Numbers

- ~4600 lines Rust
- 28 integration tests, 8 JSON scenarios
- 11 git commits
- 0 compiler warnings

---

## Roadmap

### Phase 1: Combat & Interaction (next)

**Goal: entities can fight and affect each other**

- [ ] Melee combat: rigid entity AABB overlap → damage exchange
- [ ] Health bars in terminal render (colored indicator above entity)
- [ ] Death → ragdoll → corpse decomposition (flesh cells drop into grid over time)
- [ ] Projectile system: thrown objects (arrows, fireballs) as lightweight rigid bodies
- [ ] Material interaction with entities: entity walks through fire → ignites, acid → dissolves
- [ ] Knockback: damage applies velocity impulse to rigid body center
- [ ] Goblin AI: move toward player, attack when adjacent, flee when low HP

**Tests needed:**
- Melee damage between two entities
- Knockback direction correctness
- Corpse decomposition produces flesh cells in grid
- Projectile travels and deals damage on hit
- Goblin AI moves toward player

### Phase 2: World & Exploration

**Goal: explorable world with depth and variety**

- [ ] Chunk system: world divided into chunks (64x64), only active chunks simulated
- [ ] Chunk persistence: save/load chunks to disk
- [ ] Vertical descent: stairs/holes between depth levels
- [ ] Biomes: grassland, cave, lava cavern, ice, fungus forest — each with material palette
- [ ] Procedural dungeon generation: rooms, corridors, traps
- [ ] Camera zoom: +/- keys to change viewport scale (more or fewer cells visible)
- [ ] Minimap: ASCII overview of explored area
- [ ] Day/night cycle: ambient light affects rendering (dimmer at night)

**Tests needed:**
- Chunk save/load roundtrip preserves state
- Entity crossing chunk boundary continues correctly
- Dungeon generation produces connected rooms
- Biome materials match expected palette

### Phase 3: RPG Layer

**Goal: character progression, inventory, abilities**

- [ ] Stats: strength, agility, toughness, willpower — affect damage, speed, HP, etc.
- [ ] Inventory system: items as data structs, pick up by walking over, drop with key
- [ ] Equipment: weapon affects melee damage/range, armor affects damage reduction
- [ ] Items in world: weapons, potions, scrolls, food — rendered as distinct ASCII chars
- [ ] Mutations (Caves of Qud style): modify entity properties
  - "Silicon skin" → entity material becomes Stone, immune to acid
  - "Flame body" → entity emits fire cells, immune to fire
  - "Liquid form" → entity can squeeze through 1-cell gaps
  - "Multiple arms" → extra attack, can hold more items
- [ ] XP and leveling: kill entities → gain XP → level up → choose mutation
- [ ] Skills: active abilities on cooldown (dash, stomp, material blast)
- [ ] Status effects: burning, poisoned, frozen, bleeding — each with tick effect
- [ ] Dialogue: talk to NPCs, simple text tree

**Tests needed:**
- Stat modifiers affect combat correctly
- Item pickup/drop maintains inventory integrity
- Mutation changes entity material/properties
- XP accumulation triggers level up
- Status effect ticks deal correct damage

### Phase 4: Vulkan Renderer

**Goal: 60+ FPS windowed rendering with GPU**

- [ ] ash (Vulkan) bootstrap: instance, device, swapchain, render pass
- [ ] Glyph atlas: DejaVu Sans Mono rasterized at startup via fontdue
- [ ] Instanced rendering: one draw call for all visible cells
- [ ] Persistent mapped buffer for instance data
- [ ] Dirty cell tracking: only update changed cells in instance buffer
- [ ] Camera: smooth follow, zoom levels
- [ ] Post-processing: subtle bloom for lava/fire, vignette
- [ ] `--mode auto`: try Vulkan, fallback to terminal
- [ ] Single binary: font embedded via include_bytes!

**Tests needed:**
- Vulkan init doesn't crash on supported hardware
- Render output matches terminal render for same state
- Frame time < 16ms with full viewport

### Phase 5: Content & Polish

**Goal: playable vertical slice**

- [ ] Factions: goblins, skeletons, slimes, trolls — each with AI behavior
- [ ] Boss entity: large rigid body (10x10), multiple attack patterns
- [ ] Books/readable items: lore text displayed in terminal
- [ ] Crafting: combine materials to create new ones (water + dirt = mud)
- [ ] Sound: procedural audio via terminal bell or optional ALSA
- [ ] Save/load: full game state to file (grid + entities + player + inventory)
- [ ] Death screen: stats summary, cause of death
- [ ] Tutorial: first-time controls overlay
- [ ] Difficulty scaling: deeper levels = stronger enemies

### Phase 6: Advanced Physics

**Goal: deeper Noita-style material simulation**

- [ ] Pressure: liquids have pressure, flow through pipes and U-bends
- [ ] Temperature gradient: heat radiates, materials melt/freeze at thresholds
- [ ] Electricity: conductive materials carry current, shocks entities
- [ ] Explosions: rapid gas expansion, creates fire + destroys terrain
- [ ] Structural integrity: stone/wood can collapse under load
- [ ] GPU compute: cellular automaton on Vulkan compute shader for large worlds
- [ ] Fluid simulation: proper Navier-Stokes for water instead of CA approximation

**Tests needed:**
- Pressure equalizes in connected containers
- Heat propagates through conductive materials
- Electricity follows conductive path
- Explosion destroys terrain in radius
- Collapse triggers when support removed

---

## Architecture Decisions

### Locked

| Decision | Rationale |
|----------|-----------|
| Text grid as source of truth | AI-observable, dual renderer, single state |
| Rust + crossterm + ash | Zero-cost, memory safety, explicit GPU control |
| Fixed 60Hz timestep | Deterministic replay, consistent physics |
| AABB for rigid, Verlet for ragdoll | Simple, no tunneling for rigid; expressive for ragdoll |
| Seeded RNG for determinism | Replay system, reproducible tests |
| JSON pipe protocol | Any AI agent can connect, no vision needed |
| Single binary with embedded font | Portable, no external assets |

### Open Questions

| Question | Options | When to decide |
|----------|---------|----------------|
| Turn-based vs real-time | Currently real-time 60Hz. Qud is turn-based. Hybrid? | Phase 3 |
| World topology | Single deep shaft vs branching dungeon vs open world | Phase 2 |
| Save format | Binary (compact) vs JSON (debuggable) vs RON | Phase 5 |
| Multiplayer | No. But pipe protocol could enable AI vs AI | Never (single-player) |
| Modding | Data-driven materials from JSON/TOML? | Phase 3 |

---

## File Structure (current + planned)

```
src/
  main.rs              # CLI entry point
  lib.rs               # Library root
  game.rs              # Game loop, world gen, entity management
  input.rs             # Keyboard input → Action enum
  world/
    cell.rs            # Cell struct, MaterialId enum
    material.rs        # Material properties registry
    grid.rs            # Grid (250x250), cell access
    cellular.rs        # Cellular automaton rules
    chunk.rs           # [Phase 2] chunk system
    worldgen.rs        # [Phase 2] procedural generation
  physics/
    verlet.rs          # Verlet integrator, constraints
    collision.rs       # AABB-vs-grid collision (used by ragdoll)
    projectile.rs      # [Phase 1] lightweight projectiles
  entity/
    entity.rs          # Entity struct, rigid/ragdoll, build_humanoid
    player.rs          # Player controller
    ai.rs              # [Phase 1] goblin AI
    inventory.rs       # [Phase 3] items and equipment
    stats.rs           # [Phase 3] character stats
    mutations.rs       # [Phase 3] mutation system
  render/
    mod.rs             # Renderer trait
    terminal.rs        # Terminal renderer (ANSI)
    vulkan.rs          # [Phase 4] Vulkan renderer
  ai/
    session.rs         # GameSession wrapper for AI/testing
    state.rs           # JSON state export
    action.rs          # AiAction enum
    protocol.rs        # JSON pipe protocol
    replay.rs          # Record/playback
    scenario.rs        # JSON test scenarios
tests/                 # 28 integration tests
scenarios/             # 8 JSON scenarios
assets/
  DejaVuSansMono.ttf   # Embedded font for Vulkan renderer
```

---

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| CA step (250x250) | < 1ms | ~0.5ms |
| Rigid entity update | < 0.5ms per entity | ~0.2ms |
| Terminal render frame | < 5ms | ~2ms (diff-based) |
| Vulkan render frame | < 16ms (60 FPS) | N/A |
| Pipe protocol latency | < 1ms per command | ~0.1ms |
| Binary size (release) | < 10MB | ~6MB (debug, no Vulkan) |

---

## Testing Strategy

| Layer | Method | Count |
|-------|--------|-------|
| Material physics | Rust integration tests | 15 |
| Entity physics | Rust integration tests | 8 |
| AI/replay | Rust integration tests | 4 |
| JSON scenarios | Declarative test files | 8 |
| Manual playtest | Terminal mode | As needed |
| AI playtest | Pipe protocol + agent | Future |

**Priority: every new feature gets tests before merge.**

---

## Release Milestones

| Milestone | Content | Target |
|-----------|---------|--------|
| 0.1 (done) | Core engine: CA, rigid, ragdoll, terminal, AI pipe | June 2026 |
| 0.2 | Combat, goblin AI, projectiles, corpse decomposition | July 2026 |
| 0.3 | Chunks, biomes, dungeon gen, camera zoom | August 2026 |
| 0.4 | RPG layer: stats, inventory, mutations, XP | October 2026 |
| 0.5 | Vulkan renderer, save/load, polish | December 2026 |
| 1.0 | Full vertical slice: content, balance, death screen | Q1 2027 |
