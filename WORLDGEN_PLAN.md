# World Generator Plan

## Goal
Procedural world generation with depth-based biomes, rooms, corridors, and randomized features.

## Current State
- Implemented in `src/world/worldgen.rs`
- Depth-based dispatch: surface (1-3), caves (4-6), BSP dungeon (7+)
- Randomized features, pools, trees, walls, rooms, corridors, traps
- World size: 2048x2048 for main game; 250x250 for tests/AI
- Seed-based generation with `Game::seed`
- Per-chunk world cache in `cache/worlds/<seed>/depth_<N>/`
- Tests in `tests/worldgen.rs` and `tests/large_world.rs` (ignored, slow)

## Algorithms Researched

### BSP (Binary Space Partitioning)
- Recursively divide space into rectangles
- Place room in each leaf node
- Connect siblings with corridors
- Guarantees no overlaps
- **Use for: dungeon rooms (depth 7+)**

### Cellular Automata (4-5 rule)
- Fill grid with ~45% random walls
- 5 iterations: wall if >=4 neighbors are walls, else empty
- Produces organic cave shapes
- Flood fill to verify connectivity
- **Use for: caves (depth 4-6, and underground at depth 1-3)**

### Drunkard's Walk
- Random walk digs tunnels through solid rock
- Creates winding cave-like paths
- **Use for: tunnels connecting rooms**

### Brogue Room Accretion
- Start with one room, attach new rooms to existing structure
- Inherently connected (tree structure)
- Room templates: rectangle, CA blob, circle
- **Use for: room placement strategy**

### Rooms and Mazes (Bob Nystrom)
- Place rooms → fill gaps with maze → connect → remove dead ends
- **Inspirational, not directly used**

### Noita — Herringbone Wang Tiles
- Pre-made chunks laid in herringbone pattern
- Randomized contents within chunks
- **Too complex for now, possible future enhancement**

## Architecture

### New module: `src/world/worldgen.rs`

```
WorldGenerator
├── rng: &mut CellularAutomaton
├── generate(grid, depth)              — main entry point
├── generate_surface(grid, depth)      — depth 1-3: terrain + caves + features
├── generate_caves(grid, depth)        — depth 4-6: full underground caves
├── generate_dungeon(grid, depth)      — depth 7+: BSP rooms + corridors
│
├── Surface sub-methods:
│   ├── terrain_noise(x, depth)        — multi-octave surface height
│   ├── fill_terrain(grid, depth)      — fill dirt/stone/grass by depth
│   ├── carve_underground_caves(grid)  — CA caves below surface
│   ├── place_trees(grid, count)       — random tree placement
│   ├── place_pools(grid, count, types)— random liquid pools
│   ├── place_sand_dunes(grid, count)  — sand piles
│   └── place_walls(grid, count)       — stone wall obstacles
│
├── Cave sub-methods:
│   ├── ca_caves(grid, fill_prob, iterations) — cellular automata
│   ├── flood_fill_largest(grid)       — find largest connected region
│   ├── seal_small_regions(grid)       — fill disconnected caves
│   └── place_underground_pools(grid)  — lava/acid in caves
│
├── Dungeon sub-methods:
│   ├── bsp_split(rect, depth)         — recursive space partitioning
│   ├── place_room(grid, rect)         — carve room interior
│   ├── connect_rooms(grid, rooms)     — L-shaped corridors
│   ├── place_doors(grid, rooms)       — door at room entrances
│   └── place_traps(grid, rooms)       — acid/fire traps in rooms
│
└── Shared:
    ├── place_stairs(grid, depth)      — stairs in appropriate location
    └── place_items(game, rooms)       — items in rooms/on surface
```

## Depth-based Generation

| Depth | Type | Surface | Features | Algorithm |
|-------|------|---------|----------|-----------|
| 1-3 | Surface | Grass/dirt | Trees, water pools, sand dunes, stone walls, underground CA caves | Multi-octave noise + CA |
| 4-6 | Caves | Stone/dirt | Large CA caves, lava pools, acid pools, stalactites | Cellular automata (4-5 rule) |
| 7+ | Dungeon | Stone | BSP rooms (5x3 to 12x8), corridors, traps, stairs | BSP + corridor connection |

## Implementation Details

### Surface terrain (depth 1-3)
- Multi-octave sine noise: `base + detail + micro`
- Amplitude: 4-8 cells variation
- Surface material: grass at depth 1, dirt at 2-3
- Below surface: dirt for 8 cells, then stone
- Border: stone walls

### CA cave generation (depth 4-6)
1. Fill entire grid with stone
2. Random fill ~45% as empty (cave candidate)
3. Run 5 iterations of 4-5 rule:
   - Cell becomes wall if >=4 of 8 neighbors are walls
   - Cell becomes empty if <4 neighbors are walls
4. Flood fill from center, find largest connected region
5. Seal all cells not in largest region (fill with stone)
6. Place lava/acid pools in random empty areas
7. Place stalactites (stone pillars) in random positions

### BSP dungeon (depth 7+)
1. Start with full grid as stone
2. Recursively split into 2 halves (alternate H/V)
3. Stop when area < min_room_size (15x10)
4. In each leaf: place room (smaller than partition, centered)
5. Connect sibling rooms with L-shaped corridor (2 wide)
6. Place stairs in the deepest/farthest room
7. Place items in 2-3 random rooms
8. Place traps (acid pockets) in 1-2 rooms

### Feature placement (all depths)
- All positions via RNG, not hardcoded
- Pool count: 2 + depth/2
- Pool radius: 4-10 cells
- Pool types by depth:
  - 1-3: water, sand
  - 4-6: lava, acid, water
  - 7+: acid (traps in rooms)
- Tree count: 3-7 (surface only)
- Wall count: 1-3 (surface only)

## Files

| File | Change |
|------|--------|
| `src/world/worldgen.rs` | New module — all generation logic |
| `src/world/mod.rs` | Add `pub mod worldgen` |
| `src/game.rs` | `init_world()` calls `WorldGenerator::generate()` |
| `tests/worldgen.rs` | New tests |

## Tests
- Stairs exist after generation at any depth
- At least 3 distinct materials present
- Player spawn position is not inside solid
- Different depths produce different structures
- Depth 7+ has rooms (empty regions > 5x3)
- CA caves are connected (flood fill test)

## Implementation Order
1. Create `worldgen.rs` with `WorldGenerator` struct and `generate()` dispatch
2. Implement surface generation (depth 1-3) — move existing code, add RNG
3. Implement CA cave generation (depth 4-6)
4. Implement BSP dungeon generation (depth 7+)
5. Integrate into `game.rs::init_world()`
6. Write tests
7. Run all tests + scenarios
8. Push
