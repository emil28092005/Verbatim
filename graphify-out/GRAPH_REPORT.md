# Graph Report - .  (2026-06-21)

## Corpus Check
- Corpus is ~33,231 words - fits in a single context window. You may not need a graph.

## Summary
- 690 nodes · 1218 edges · 47 communities (44 shown, 3 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 32 edges (avg confidence: 0.86)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Vulkan Core Setup|Vulkan Core Setup]]
- [[_COMMUNITY_Terminal Renderer & Game Loop|Terminal Renderer & Game Loop]]
- [[_COMMUNITY_Graphics Vulkan Renderer|Graphics Vulkan Renderer]]
- [[_COMMUNITY_AI Pipe Protocol|AI Pipe Protocol]]
- [[_COMMUNITY_AI Game Session|AI Game Session]]
- [[_COMMUNITY_CLI Entry & GPU Mode|CLI Entry & GPU Mode]]
- [[_COMMUNITY_AI State Export|AI State Export]]
- [[_COMMUNITY_Terminal Input Handler|Terminal Input Handler]]
- [[_COMMUNITY_Entity System|Entity System]]
- [[_COMMUNITY_Cellular Automaton|Cellular Automaton]]
- [[_COMMUNITY_Replay System|Replay System]]
- [[_COMMUNITY_Edge Case Tests|Edge Case Tests]]
- [[_COMMUNITY_AI Actions & Grid|AI Actions & Grid]]
- [[_COMMUNITY_Material Physics Tests|Material Physics Tests]]
- [[_COMMUNITY_Player Controller|Player Controller]]
- [[_COMMUNITY_Cell & Material Types|Cell & Material Types]]
- [[_COMMUNITY_Material Interaction Tests|Material Interaction Tests]]
- [[_COMMUNITY_Player Control Tests|Player Control Tests]]
- [[_COMMUNITY_AI Agent Roadmap|AI Agent Roadmap]]
- [[_COMMUNITY_Verlet Physics Solver|Verlet Physics Solver]]
- [[_COMMUNITY_Render & Phase Roadmap|Render & Phase Roadmap]]
- [[_COMMUNITY_Collision Robustness Tests|Collision Robustness Tests]]
- [[_COMMUNITY_Render Mode & UI Roadmap|Render Mode & UI Roadmap]]
- [[_COMMUNITY_Window Input (winit)|Window Input (winit)]]
- [[_COMMUNITY_Determinism Tests|Determinism Tests]]
- [[_COMMUNITY_Ragdoll Death Tests|Ragdoll Death Tests]]
- [[_COMMUNITY_Grid Collision Resolver|Grid Collision Resolver]]
- [[_COMMUNITY_Material Registry|Material Registry]]
- [[_COMMUNITY_Entity Types & Game Loop|Entity Types & Game Loop]]
- [[_COMMUNITY_Adaptive Viewport|Adaptive Viewport]]
- [[_COMMUNITY_Headless Dump & Combat Plan|Headless Dump & Combat Plan]]
- [[_COMMUNITY_Entity Damage Tests|Entity Damage Tests]]
- [[_COMMUNITY_Entity Movement Tests|Entity Movement Tests]]
- [[_COMMUNITY_Lava Physics Tests|Lava Physics Tests]]
- [[_COMMUNITY_Sand Physics Tests|Sand Physics Tests]]
- [[_COMMUNITY_Acid Physics Tests|Acid Physics Tests]]
- [[_COMMUNITY_Water Physics Tests|Water Physics Tests]]
- [[_COMMUNITY_Ragdoll Physics Plan|Ragdoll Physics Plan]]
- [[_COMMUNITY_Per-Cell Color Plan|Per-Cell Color Plan]]
- [[_COMMUNITY_Game Run & Input|Game Run & Input]]
- [[_COMMUNITY_Slope Stepping|Slope Stepping]]
- [[_COMMUNITY_Renderer Trait|Renderer Trait]]

## God Nodes (most connected - your core abstractions)
1. `VulkanRenderer` - 37 edges
2. `GraphicsRenderer` - 31 edges
3. `GameSession` - 26 edges
4. `Game` - 24 edges
5. `Entity` - 20 edges
6. `CellularAutomaton` - 19 edges
7. `InputHandler` - 18 edges
8. `Result` - 17 edges
9. `String` - 17 edges
10. `setup()` - 17 edges

## Surprising Connections (you probably didn't know these)
- `pipe render mode` --semantically_similar_to--> `AI pipe protocol (JSON stdin/stdout)`  [INFERRED] [semantically similar]
  AGENTS.md → PLAN.md
- `Game::fixed_update()` --semantically_similar_to--> `Fixed 60Hz timestep`  [INFERRED] [semantically similar]
  AGENTS.md → PLAN.md
- `Player` --semantically_similar_to--> `Player entity (headless dump)`  [INFERRED] [semantically similar]
  AGENTS.md → headless_dump.txt
- `rigid entity type` --semantically_similar_to--> `Rigid entities (AABB + slope stepping)`  [INFERRED] [semantically similar]
  AGENTS.md → PLAN.md
- `ragdoll entity type` --semantically_similar_to--> `Ragdoll corpses (Verlet)`  [INFERRED] [semantically similar]
  AGENTS.md → PLAN.md

## Import Cycles
- 1-file cycle: `src/ai/action.rs -> src/ai/action.rs`
- 1-file cycle: `src/world/grid.rs -> src/world/grid.rs`
- 1-file cycle: `src/ai/protocol.rs -> src/ai/protocol.rs`
- 1-file cycle: `src/ai/replay.rs -> src/ai/replay.rs`
- 1-file cycle: `src/ai/scenario.rs -> src/ai/scenario.rs`
- 1-file cycle: `src/ai/session.rs -> src/ai/session.rs`
- 1-file cycle: `src/ai/state.rs -> src/ai/state.rs`
- 1-file cycle: `src/game.rs -> src/game.rs`
- 1-file cycle: `src/entity/player.rs -> src/entity/player.rs`
- 1-file cycle: `src/render/terminal.rs -> src/render/terminal.rs`
- 1-file cycle: `src/input.rs -> src/input.rs`
- 1-file cycle: `src/main.rs -> src/main.rs`
- 1-file cycle: `src/physics/collision.rs -> src/physics/collision.rs`
- 1-file cycle: `src/physics/verlet.rs -> src/physics/verlet.rs`
- 1-file cycle: `src/render/graphics.rs -> src/render/graphics.rs`
- 1-file cycle: `src/render/vulkan.rs -> src/render/vulkan.rs`
- 1-file cycle: `src/render/window_input.rs -> src/render/window_input.rs`
- 1-file cycle: `src/world/cellular.rs -> src/world/cellular.rs`
- 1-file cycle: `src/world/material.rs -> src/world/material.rs`
- 1-file cycle: `tests/collision_robust.rs -> tests/collision_robust.rs`

## Hyperedges (group relationships)
- **Game loop fixed_update pipeline: CA step → rigid update → ragdoll update → damage** —  [INFERRED]
- **Three render modes read the same Grid + EntityManager (single source of truth)** —  [INFERRED]
- **GpuRenderer trait unifies VulkanRenderer + GraphicsRenderer behind run_gpu_mode<R>() with identical event loops** —  [INFERRED]

## Communities (47 total, 3 thin omitted)

### Community 0 - "Vulkan Core Setup"
Cohesion: 0.10
Nodes (61): BufferUsageFlags, c_char, DescriptorPool, DescriptorSet, DescriptorSetLayout, Format, Image, MemoryPropertyFlags (+53 more)

### Community 1 - "Terminal Renderer & Game Loop"
Cohesion: 0.07
Nodes (19): CellularAutomaton, Duration, InputHandler, Player, R, TerminalRenderer, Renderer, Game (+11 more)

### Community 2 - "Graphics Vulkan Renderer"
Cohesion: 0.07
Nodes (31): ColorInstance, GraphicsRenderer, PushConstants, Arc, Buffer, CommandBuffer, CommandPool, Device (+23 more)

### Community 3 - "AI Pipe Protocol"
Cohesion: 0.12
Nodes (31): Command, handle_command(), Response, run_pipe_protocol(), Assertion, AssertionResult, check_assertion(), format_results() (+23 more)

### Community 4 - "AI Game Session"
Cohesion: 0.09
Nodes (13): GameSession, Default, ReplayRecorder, AiAction, CellInfo, EntityInfo, Game, GameState (+5 more)

### Community 5 - "CLI Entry & GPU Mode"
Cohesion: 0.15
Nodes (20): Cli, dump_view(), GpuRenderer, main(), player_info(), Arc, EntityManager, Game (+12 more)

### Community 6 - "AI State Export"
Cohesion: 0.13
Nodes (23): build_game_state(), CellInfo, entity_info(), entity_kind_name(), EntityInfo, GameState, material_from_name(), parse_entity_kind() (+15 more)

### Community 7 - "Terminal Input Handler"
Cohesion: 0.13
Nodes (15): Event, JoinHandle, Receiver, Action, HeldKey, HeldState, InputHandler, MaterialBrush (+7 more)

### Community 8 - "Entity System"
Cohesion: 0.15
Nodes (9): Constraint, Entity, EntityKind, EntityManager, EntityId, Option, Self, SubBody (+1 more)

### Community 9 - "Cellular Automaton"
Cohesion: 0.29
Nodes (4): Grid, MaterialId, Self, CellularAutomaton

### Community 10 - "Replay System"
Cohesion: 0.13
Nodes (11): ReplayEvent, ReplayPlayer, ReplayRecorder, ReplayRecording, ReplayEvent, AiAction, GameSession, Result (+3 more)

### Community 11 - "Edge Case Tests"
Cohesion: 0.16
Nodes (17): center_camera_on_player(), clear_region_removes_all_materials(), entity_at_right_boundary(), entity_at_world_boundary_stays_in_bounds(), fill_rect_creates_correct_material_count(), find_material_locates_existing(), find_material_returns_none_for_absent(), goblin_has_correct_max_health() (+9 more)

### Community 12 - "AI Actions & Grid"
Cohesion: 0.18
Nodes (7): AiAction, Cell, Game, MaterialId, Self, Vec, Grid

### Community 13 - "Material Physics Tests"
Cohesion: 0.22
Nodes (17): bone_is_static(), dirt_is_solid(), fire_dies_over_time(), fire_ignites_flesh(), fire_ignites_grass(), fire_ignites_wood(), grass_is_solid(), lava_initial_temp_is_high() (+9 more)

### Community 14 - "Player Controller"
Cohesion: 0.21
Nodes (6): Player, Entity, EntityId, EntityManager, Option, Self

### Community 15 - "Cell & Material Types"
Cohesion: 0.17
Nodes (4): Self, Cell, MaterialId, rand_u8()

### Community 16 - "Material Interaction Tests"
Cohesion: 0.26
Nodes (14): acid_dissolves_dirt(), acid_dissolves_grass(), acid_does_not_dissolve_empty(), fire_does_not_ignite_stone(), fire_does_not_ignite_water(), fire_spreads_through_wood_line(), lava_and_water_produce_both_steam_and_stone(), lava_cools_to_stone_eventually() (+6 more)

### Community 17 - "Player Control Tests"
Cohesion: 0.26
Nodes (14): continuous_movement_does_not_fall_through_floor(), jump_goes_up_then_falls_back(), jump_while_airborne_does_nothing(), move_left_changes_x_position(), move_left_then_right_cancels(), move_right_changes_x_position(), player_health_stays_full_without_damage(), player_not_on_ground_while_jumping() (+6 more)

### Community 18 - "AI Agent Roadmap"
Cohesion: 0.18
Nodes (14): AiAction, GameSession, pipe render mode, AI pipe protocol (JSON stdin/stdout), Seeded RNG determinism, LLM agent (Ollama/Llama/Qwen), Phase 7: AI Agent Integration, Phase 8: Web Arena & Training Pipeline (+6 more)

### Community 19 - "Verlet Physics Solver"
Cohesion: 0.22
Nodes (5): Constraint, SubBody, VerletSolver, MaterialId, Self

### Community 20 - "Render & Phase Roadmap"
Cohesion: 0.15
Nodes (11): Chunk system (64x64), Cross-platform support (Win/Linux/macOS), Multi-layer world architecture, Phase 2: World & Exploration, Phase 3: RPG Layer, Phase 4: Render Modes, Phase 4b: Graphics layers (lighting/particles/textures), Phase 5: Content & Polish (+3 more)

### Community 21 - "Collision Robustness Tests"
Cohesion: 0.27
Nodes (10): entity_collision_with_dirt_wall(), player_blocked_by_ceiling(), player_blocked_by_left_wall(), player_blocked_by_right_wall(), player_blocked_by_two_walls_both_sides(), player_does_not_stick_to_wall(), player_slides_along_wall(), player_squeezes_through_gap() (+2 more)

### Community 22 - "Render Mode & UI Roadmap"
Cohesion: 0.22
Nodes (11): EntityManager, Grid (250x250) source of truth, ascii render mode, graphics render mode, terminal render mode, ASCII renderer (Vulkan glyph atlas), Graphics renderer (Vulkan colored cells), Phase 1.5: UI Layer (+3 more)

### Community 23 - "Window Input (winit)"
Cohesion: 0.20
Nodes (7): ElementState, HashSet, PhysicalKey, WindowInput, KeyCode, Option, Self

### Community 24 - "Determinism Tests"
Cohesion: 0.22
Nodes (3): determinism_with_spawn_and_damage(), GameSession, setup()

### Community 25 - "Ragdoll Death Tests"
Cohesion: 0.38
Nodes (9): corpse_exists_in_world(), damage_reduces_health_progressively(), death_transitions_rigid_to_ragdoll(), player_death_becomes_corpse(), ragdoll_bodies_stay_near_each_other(), ragdoll_falls_after_death(), GameSession, setup() (+1 more)

### Community 26 - "Grid Collision Resolver"
Cohesion: 0.46
Nodes (6): apply_liquid_drag(), CollisionResult, resolve_grid_collision(), Grid, Self, SubBody

### Community 27 - "Material Registry"
Cohesion: 0.43
Nodes (4): MaterialId, Self, Material, MaterialRegistry

### Community 28 - "Entity Types & Game Loop"
Cohesion: 0.29
Nodes (7): cellular entity type, CellularAutomaton, Game::fixed_update(), rigid entity type, Fixed 60Hz timestep, 14 CA Materials, Rigid entities (AABB + slope stepping)

### Community 29 - "Adaptive Viewport"
Cohesion: 0.33
Nodes (7): check_resize(), GpuRenderer trait, GraphicsRenderer, run_gpu_mode<R>(), VulkanRenderer, WindowInput, Adaptive viewport

### Community 30 - "Headless Dump & Combat Plan"
Cohesion: 0.33
Nodes (6): Player, vector movement convention, Goblin entity (headless dump), Grid ASCII rendering (headless dump), Player entity (headless dump), Phase 1: Combat & Interaction

### Community 31 - "Entity Damage Tests"
Cohesion: 0.52
Nodes (6): entity_blocked_by_stone(), entity_dies_becomes_corpse(), entity_on_fire_takes_damage_over_time(), entity_takes_lava_damage(), GameSession, setup_empty()

### Community 32 - "Entity Movement Tests"
Cohesion: 0.48
Nodes (5): player_blocked_by_stone_wall(), player_can_move_right(), player_survives_fall(), GameSession, setup_empty()

### Community 33 - "Lava Physics Tests"
Cohesion: 0.52
Nodes (6): lava_flows_down(), lava_ignites_grass(), lava_ignites_wood(), lava_plus_water_makes_steam(), GameSession, setup_empty()

### Community 34 - "Sand Physics Tests"
Cohesion: 0.52
Nodes (6): GameSession, sand_displaces_water(), sand_does_not_fall_through_stone(), sand_falls_down(), sand_piles_on_stone(), setup_empty()

### Community 35 - "Acid Physics Tests"
Cohesion: 0.60
Nodes (5): acid_dissolves_wood(), acid_does_not_dissolve_stone(), acid_flows_down(), GameSession, setup_empty()

### Community 36 - "Water Physics Tests"
Cohesion: 0.60
Nodes (5): GameSession, setup_empty(), water_does_not_pass_through_stone_wall(), water_flows_down(), water_spreads_sideways()

### Community 37 - "Ragdoll Physics Plan"
Cohesion: 0.50
Nodes (5): Constraint, ragdoll entity type, SubBody, VerletSolver, Ragdoll corpses (Verlet)

### Community 39 - "Per-Cell Color Plan"
Cohesion: 0.67
Nodes (4): Cell struct, inline cell colors convention, MaterialRegistry, Per-cell color (reality layer)

## Knowledge Gaps
- **106 isolated node(s):** `String`, `CellInfo`, `Vec`, `EntityInfo`, `ScenarioResult` (+101 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **3 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Cell` connect `AI Actions & Grid` to `Cellular Automaton`?**
  _High betweenness centrality (0.011) - this node is a cross-community bridge._
- **Why does `GameSession` connect `AI Game Session` to `AI State Export`?**
  _High betweenness centrality (0.011) - this node is a cross-community bridge._
- **What connects `String`, `CellInfo`, `Vec` to the rest of the system?**
  _108 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Vulkan Core Setup` be split into smaller, more focused modules?**
  _Cohesion score 0.09525899912203688 - nodes in this community are weakly interconnected._
- **Should `Terminal Renderer & Game Loop` be split into smaller, more focused modules?**
  _Cohesion score 0.06956521739130435 - nodes in this community are weakly interconnected._
- **Should `Graphics Vulkan Renderer` be split into smaller, more focused modules?**
  _Cohesion score 0.06543385490753911 - nodes in this community are weakly interconnected._
- **Should `AI Pipe Protocol` be split into smaller, more focused modules?**
  _Cohesion score 0.12063492063492064 - nodes in this community are weakly interconnected._