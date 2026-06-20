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

### Phase 4: Vulkan Renderer + Graphics Over ASCII

**Goal: 60+ FPS windowed rendering with GPU, graphics layered over ASCII grid**

- [ ] ash (Vulkan) bootstrap: instance, device, swapchain, render pass
- [ ] Glyph atlas: DejaVu Sans Mono rasterized at startup via fontdue
- [ ] Instanced rendering: one draw call for all visible cells
- [ ] Persistent mapped buffer for instance data
- [ ] Dirty cell tracking: only update changed cells in instance buffer
- [ ] Camera: smooth follow, zoom levels
- [ ] `--mode auto`: try Vulkan, fallback to terminal
- [ ] Single binary: font embedded via include_bytes!

**Graphics layers over ASCII (Phase 4b):**
- [ ] Lighting pass: compute shader calculates light grid from sources (lava, fire, torches)
  - Materials emit light with color/intensity
  - Walls cast shadows (ray-march in compute)
  - Light grid modulates cell brightness in render
- [ ] Particle system: GPU particles positioned relative to grid cells
  - Fire sparks, water splashes, smoke trails, blood
  - Particle lifetime + physics (gravity, wind)
- [ ] Procedural material textures: per-cell texture instead of flat color
  - Stone: noise pattern, cracks
  - Water: animated wave distortion
  - Lava: flowing magma texture, glow
  - Wood: grain pattern
- [ ] Post-processing: bloom (bright materials glow), vignette, optional CRT curvature
- [ ] Ambient effects: heat shimmer above lava, dust motles in air, screen shake on explosions

**Terminal mode stays pure ASCII. Vulkan mode = ASCII + graphics layers.**

**Tests needed:**
- Vulkan init doesn't crash on supported hardware
- Render output matches terminal render for same state (cell positions/colors)
- Frame time < 16ms with full viewport + lighting + particles
- Lighting grid updates when light sources change
- Particle count scales with active fire/lava cells

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

- [ ] Multi-layer world: separate grid layers for material, temperature, pressure, gas/air, light
  - Air layer: gas flow, ventilation in caves, gas accumulates at ceiling, displaced by fire
  - Pressure layer: liquids have pressure, flow through pipes and U-bends
  - Temperature layer: proper heat diffusion, materials melt/freeze at thresholds
  - Light layer: ray-cast from sources (lava, fire, torch), affects rendering
  - Layers interact: fire heats temp layer → temp melts material → material releases gas
- [ ] Electricity: conductive materials carry current, shocks entities
- [ ] Explosions: rapid gas expansion, creates fire + destroys terrain
- [ ] Structural integrity: stone/wood can collapse under load
- [ ] GPU compute: cellular automaton on Vulkan compute shader for large worlds
- [ ] Fluid simulation: proper Navier-Stokes for water instead of CA approximation

**Architecture: `World { layers: Vec<GridLayer> }` — each layer is a separate grid updated by its own rules, with cross-layer interactions.**

**Tests needed:**
- Pressure equalizes in connected containers
- Heat propagates through conductive materials
- Gas flows upward, accumulates at ceiling
- Electricity follows conductive path
- Explosion destroys terrain in radius
- Collapse triggers when support removed
- Layer interaction: fire → temp rise → material melt

### Phase 7: AI Agent Integration

**Goal: local neural network plays Verbatim as an agent**

- [ ] LLM agent: local model (Ollama/Llama/Qwen) connects via pipe protocol
  - Reads JSON state (ASCII view + structured data)
  - Reasons in text, sends JSON actions
  - Good for testing mechanics, exploration, debug
- [ ] RL agent: trained policy network (PyTorch)
  - State as tensor (material grid + entity positions + HP)
  - Action as discrete output (move, attack, use ability)
  - Fast inference, real-time play
  - Requires training data (see Phase 8)
- [ ] Agent observation format: compact binary state tensor for RL (not JSON)
- [ ] Agent action batch mode: multiple actions per pipe message for throughput
- [ ] Agent recording: save (state, action, reward) tuples for offline training
- [ ] Agent vs agent: two pipe connections, competitive play

**Tests needed:**
- LLM agent can init, observe, act, quit via pipe
- RL state tensor matches grid state
- Recording produces valid training data format
- Agent vs agent game completes with winner

### Phase 8: Web Arena & Training Pipeline

**Goal: browser-based multiplayer arena for human + AI training data**

- [ ] Headless game server: Rust + tokio, authoritative simulation, WebSocket API
- [ ] WASM render port: game renders in browser via Canvas/WebGL, reads JSON state
- [ ] WebSocket bridge: server ↔ browser, state diffs + input commands
- [ ] Arena mode: single room, enemies, fast respawn, score timer
- [ ] Multiplayer: multiple clients connect to same server, shared world
- [ ] Recording pipeline: all player sessions recorded as (state, action, outcome) tuples
- [ ] Dataset export: recorded sessions → training data for RL agent (Phase 7)
- [ ] Leaderboard: human vs AI scores, competitive training incentive
- [ ] Spectator mode: watch AI agents fight, replay system in browser

**Architecture:**
```
Browser (WASM + Canvas)  ←WebSocket→  Rust Server (tokio + game engine)
     ↑                                    ↑
  Player input                       Pipe protocol
                                     (AI agents connect locally)
```

**Tests needed:**
- Server accepts WebSocket connections
- State sync: all clients see same world state
- WASM render matches terminal render for same state
- Recording captures all state changes
- Dataset export produces valid tensor format
- Multiple clients don't desync

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
| Multiplayer | Phase 8: web arena for AI training. Core game stays single-player | Phase 8 |
| Modding | Data-driven materials from JSON/TOML? | Phase 3 |
| Multi-layer architecture | Separate grids per layer vs interleaved in one Cell? | Phase 6 |
| RL model architecture | CNN over grid? Transformer? Hybrid? | Phase 7 |
| WASM render target | Canvas 2D vs WebGL vs WebGPU | Phase 8 |

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
    layers.rs          # [Phase 6] multi-layer world (temp, pressure, gas, light)
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
    lighting.rs        # [Phase 4b] compute shader lighting
    particles.rs       # [Phase 4b] GPU particle system
    textures.rs        # [Phase 4b] procedural material textures
  ai/
    session.rs         # GameSession wrapper for AI/testing
    state.rs           # JSON state export
    action.rs          # AiAction enum
    protocol.rs        # JSON pipe protocol
    replay.rs          # Record/playback
    scenario.rs        # JSON test scenarios
    rl_bridge.rs       # [Phase 7] tensor state export for RL agents
    recording.rs       # [Phase 7/8] (state, action, reward) recording
server/                # [Phase 8] web arena server
  server.rs            # tokio WebSocket server
  arena.rs             # arena game mode
  recording.rs         # training data collection
web/                   # [Phase 8] WASM browser client
  render.rs            # Canvas/WebGL render from JSON state
  input.rs             # browser keyboard → commands
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
| RL state export | < 0.5ms per frame | N/A |
| WebSocket state sync | < 50ms per frame | N/A |
| Binary size (release) | < 10MB | ~6MB (debug, no Vulkan) |

---

## Testing Strategy

| Layer | Method | Count |
|-------|--------|-------|
| Material physics | Rust integration tests | 15 |
| Entity physics | Rust integration tests | 8 |
| AI/replay | Rust integration tests | 4 |
| JSON scenarios | Declarative test files | 8 |
| Multi-layer physics | Rust integration tests | [Phase 6] |
| RL bridge | Rust integration tests | [Phase 7] |
| Web server | Rust integration tests | [Phase 8] |
| Manual playtest | Terminal mode | As needed |
| AI playtest | Pipe protocol + agent | [Phase 7] |

**Priority: every new feature gets tests before merge.**

---

## Release Milestones

| Milestone | Content | Target |
|-----------|---------|--------|
| 0.1 (done) | Core engine: CA, rigid, ragdoll, terminal, AI pipe | June 2026 |
| 0.2 | Combat, goblin AI, projectiles, corpse decomposition | July 2026 |
| 0.3 | Chunks, biomes, dungeon gen, camera zoom | August 2026 |
| 0.4 | RPG layer: stats, inventory, mutations, XP | October 2026 |
| 0.5 | Vulkan renderer + graphics layers (lighting, particles, textures) | December 2026 |
| 0.6 | Multi-layer world: air, pressure, temperature, light as separate grids | Feb 2027 |
| 0.7 | AI agent: LLM + RL bridge, agent recording | April 2027 |
| 0.8 | Web arena: WASM render, WebSocket server, multiplayer, training pipeline | June 2027 |
| 1.0 | Full vertical slice: content, balance, death screen, trained AI agents | Q3 2027 |
