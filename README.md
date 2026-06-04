# ternary-frontier: Exploration and discovery of unknown ternary state space

Models how agents explore and push into unknown territory in balanced ternary {-1, 0, +1} systems. The frontier is the boundary between known and unknown — this crate tracks who explores what, how risky it is, and who got there first.

## Why This Exists

In multi-agent ternary systems, agents need to discover new states and capabilities. Without a formal model of exploration, you get chaos: duplicated effort, uncharted dead zones, and no way to reward pioneers. This crate formalizes the frontier — the boundary between known and unknown — so exploration becomes structured, trackable, and incentivized.

## Core Concepts

- **Balanced ternary**: A number system using three values: -1, 0, +1 (called Neg, Zero, Pos). Each ternary coordinate is a cell in a 3×3 grid of possible states.
- **Frontier**: The set of coordinates adjacent to known territory but not yet explored. These are the "edge" of what you understand.
- **Explorer**: An agent with limited energy that moves through the state space, making discoveries.
- **DiscoveryLog**: An append-only record of what was found, by whom, and when.
- **RiskAssessment**: Evaluates the danger of frontier expansion. More unknown neighbors = higher risk. Positive ternary signals are safer; negative signals are dangerous.
- **PioneerBonus**: Rewards the first agent to discover a region. Subsequent visitors get no bonus — incentivizing rapid exploration.
- **TerritoryMap**: Tracks which agent owns which coordinates. Prevents conflicting claims.

## Quick Start

```toml
[dependencies]
ternary-frontier = "0.1"
```

```rust
use ternary_frontier::{Frontier, Explorer, DiscoveryLog, Ternary, Coord, PioneerBonus};

let mut frontier = Frontier::new();
let mut explorer = Explorer::new("scout-1");
let mut log = DiscoveryLog::new();
let mut bonuses = PioneerBonus::new();

// Explore the center of the state space
let center = Coord::new(Ternary::Zero, Ternary::Zero);
explorer.move_to(center);
explorer.discover(Ternary::Pos, &mut log, 0);
frontier.explore(center);
bonuses.record(center, &explorer.name);

println!("Known: {}/9, Pioneer bonus: {}", frontier.known_count(), bonuses.bonus("scout-1"));
```

## API Overview

| Type | Description |
|------|-------------|
| `Ternary` | A ternary value: Neg (-1), Zero (0), or Pos (+1) |
| `Coord` | A 2D coordinate in ternary space (x, y both ternary) |
| `Frontier` | Tracks known/frontier/unknown regions in the 3×3 state space |
| `Explorer` | An agent with energy, position, and discovery count |
| `DiscoveryLog` | Append-only log of all discoveries with timestamps |
| `RiskAssessment` | Evaluates risk of expanding into unknown territory |
| `PioneerBonus` | Tracks first discoverers and awards bonuses |
| `TerritoryMap` | Claim-based ownership of coordinates |

## How It Works

The `Frontier` maintains three disjoint sets: known coordinates, frontier coordinates (adjacent to known), and unknown coordinates. When a coordinate is explored, it moves from frontier or unknown into known, and its neighbors are promoted to frontier.

`RiskAssessment` uses a simple heuristic: count known vs unknown neighbors. Many known neighbors = safe expansion. Many unknown = dangerous. The ternary value at a location also factors in — positive signals are interpreted as safe, negative as hostile.

`PioneerBonus` is a first-write-wins map. The first explorer to claim a coordinate gets a bonus point; subsequent visitors get nothing. This creates an incentive to explore rapidly and broadly rather than re-treading familiar ground.

## Known Limitations

- **Fixed 3×3 grid**: The state space is always 9 coordinates. For larger grids, you'd need to extend or replace `Coord` and `Frontier`.
- **No weighted exploration**: All frontier coordinates are treated equally. There's no heuristic to prioritize promising directions.
- **Energy model is trivial**: Explorers lose 1 energy per move with no cost variance. Real systems would have terrain-dependent costs.
- **No concurrent exploration**: No synchronization primitives. If multiple threads explore simultaneously, you need external coordination.
- **Risk assessment is heuristic**: Not based on formal probability models. Tuned for gameplay-like scenarios, not safety-critical systems.

## Use Cases

- **Multi-agent room exploration**: Agents in a ternary state space discover new rooms and claim territory, with pioneer bonuses rewarding bold scouts.
- **Capability discovery**: A system that progressively discovers what operations are available, pushing the frontier of known capabilities.
- **Game world mapping**: Roguelike or exploration games where players map unknown territory in a ternary-valued world.
- **Testing coverage**: Track which code paths (mapped to ternary coordinates) have been exercised by tests.

## Ecosystem Context

Part of the SuperInstance ternary crate family. Relates to:
- `ternary-room` (rooms are the spaces being explored)
- `ternary-protocol` (exploration may use protocol messages)
- `ternary-econ` (pioneer bonuses are an economic incentive)
- `ternary-scheduling` (scheduling explorer activities over time)

## License

MIT
