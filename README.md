# lau-collab

Multiplayer collaboration layer for the Lau platform — shared worlds, player matching, co-building, agent sharing, and session logging. Kids play together, share worlds, and teach each other's agents.

## What This Does

This crate provides the data structures for collaborative play in a voxel/world-building environment:

- **Players** with identities and avatar colors
- **Shared worlds** with owner/editor/viewer permission levels
- **Collaboration events** — joins, leaves, co-builds, agent sharing, world merges
- **Session logs** that record events, track participants, and generate narrative summaries
- **Player matching** based on shared interests (tag-based)

Everything is pure data — no networking, no I/O. You drive it from your own transport layer. All types are `Serialize`/`Deserialize`.

**28 tests.**

## Key Idea

Multiplayer collaboration for a kids' world-building game. Players own worlds, invite collaborators with permission levels, and the system tracks who did what in a session log with narrative summaries. A simple interest-based matcher pairs kids who want to play the same things.

## Install

```toml
[dependencies]
lau-collab = "0.1.0"
```

Rust 2021 edition. Only dependency: `serde` with derive.

## Quick Start

### Create a shared world with collaborators

```rust
use lau_collab::*;

let mut world = SharedWorld::new("world-1", "alice");
world.invite("bob".into(), Permission::Editor);
world.invite("carol".into(), Permission::Viewer);

assert!(world.can_build("alice"));  // owner
assert!(world.can_build("bob"));    // editor
assert!(!world.can_build("carol")); // viewer only
```

### Record a session and generate a summary

```rust
use lau_collab::*;

let mut log = SessionLog::new(0, 300);
log.record(CollabEvent::PlayerJoined { player: "alice".into(), world: "w1".into() });
log.record(CollabEvent::PlayerJoined { player: "bob".into(), world: "w1".into() });
log.record(CollabEvent::CoBuild {
    players: vec!["alice".into(), "bob".into()],
    structure: "castle".into(),
    voxels: 500,
});
log.record(CollabEvent::AgentShared {
    from: "alice".into(), to: "bob".into(), agent_id: "ag-1".into(),
});
log.record(CollabEvent::PlayerLeft { player: "bob".into(), world: "w1".into() });

println!("{}", log.summary());
// 🌟 Session report (tick 0 – 300, 300 ticks):
// 🚪 2 player(s) joined the adventure.
// 👋 1 player(s) headed home.
// 🏗️ 1 co-build(s) brought ideas to life!
// 🤖 1 agent(s) were shared between friends.
// 👥 2 unique player(s): alice, bob
```

### Match players by shared interests

```rust
use lau_collab::*;

let mut matcher = CollabMatcher::new();
matcher.add_looking("alice".into(), vec!["castles".into(), "space".into()]);
matcher.add_looking("bob".into(), vec!["space".into(), "ocean".into()]);

if let Some(partner) = matcher.find_match("alice") {
    println!("Matched with {}!", partner); // "bob"
}
```

## API Reference

### Player

```rust
Player::new(id, name, avatar_color: [f64; 3], world_id) -> Player
```

Fields: `id`, `name`, `avatar_color` (RGB), `world_id`. Implements `PartialEq` and serde.

### Permission

| Variant | Can build? |
|---|---|
| `Owner` | ✅ |
| `Editor` | ✅ |
| `Viewer` | ❌ |

`Default` is `Viewer`.

### SharedWorld

| Method | Description |
|---|---|
| `new(world_id, owner)` | Create world with owner as sole collaborator. |
| `invite(player_id, permission)` | Add a collaborator with given permission. |
| `remove(player_id)` | Remove a collaborator. Cannot remove owner. Returns `bool`. |
| `is_member(player_id)` | Check membership. |
| `permission(player_id)` | Get permission level (defaults to `Viewer` for non-members). |
| `can_build(player_id)` | `true` for Owner or Editor. |
| `member_count()` | Number of collaborators (including owner). |

### CollabEvent

| Variant | Fields | Players extracted |
|---|---|---|
| `PlayerJoined` | `player`, `world` | player |
| `PlayerLeft` | `player`, `world` | player |
| `CoBuild` | `players`, `structure`, `voxels` | all players |
| `AgentShared` | `from`, `to`, `agent_id` | from + to |
| `WorldMerged` | `world_a`, `world_b`, `result` | none |

### SessionLog

| Method | Description |
|---|---|
| `new(start_tick, duration_ticks)` | Create empty log. |
| `record(event)` | Append an event. |
| `participants()` | Deduplicated player IDs from all events. |
| `events_by_type()` | Count of each event type tag. |
| `summary()` | Narrative emoji-formatted session report. |

### CollabMatcher

| Method | Description |
|---|---|
| `new()` | Empty matcher. |
| `add_looking(player_id, interests)` | Register a player looking for partners. |
| `remove_looking(player_id)` | Unregister. |
| `find_match(player_id)` | Find another player sharing at least one interest. Returns `Option<String>`. |

Implements `Default`.

## How It Works

### Permission Model

Each `SharedWorld` maintains a `HashMap<String, Permission>` mapping player IDs to access levels. The owner is always a member with `Permission::Owner` and cannot be removed. Non-members default to `Viewer` (the `Default` impl).

### Event Tracking

`CollabEvent` is an enum with five variants. Each variant knows how to extract the player IDs it references (via the private `player_ids()` method) and its type tag. `SessionLog` uses these to build participant lists and type-count summaries without any external bookkeeping.

### Session Summary

The `summary()` method counts each event type and builds a human-readable, emoji-prefixed narrative. Empty sessions get a melancholic message. This is designed for kids to see what happened in their play session.

### Player Matching

`CollabMatcher` does simple set intersection: two players match if they share at least one interest tag. It returns the first match found (no ranking). Remove a player to take them out of the pool.

## The Math

This crate is intentionally lightweight — no heavy math. The matching algorithm is O(n·k) where n is the number of looking players and k is the average number of interests per player. Participant deduplication is O(e·p) where e is events and p is average players per event.

## License

MIT
