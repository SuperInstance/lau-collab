# lau-collab

> Multiplayer collaboration layer for the Lau platform

## What This Does

Multiplayer collaboration layer for the Lau platform. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-collab
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_collab::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub struct Player 
    pub fn new(id: &str, name: &str, avatar_color: [f64; 3], world_id: &str) -> Self 
pub enum Permission 
pub struct SharedWorld 
    pub fn new(world_id: &str, owner: &str) -> Self 
    pub fn invite(&mut self, player_id: String, perm: Permission) 
    pub fn remove(&mut self, player_id: &str) -> bool 
    pub fn is_member(&self, player_id: &str) -> bool 
    pub fn permission(&self, player_id: &str) -> Permission 
    pub fn can_build(&self, player_id: &str) -> bool 
    pub fn member_count(&self) -> usize 
pub enum CollabEvent 
pub struct SessionLog 
    pub fn new(start_tick: u64, duration_ticks: u64) -> Self 
    pub fn record(&mut self, event: CollabEvent) 
    pub fn participants(&self) -> Vec<String> 
    pub fn events_by_type(&self) -> HashMap<String, usize> 
    pub fn summary(&self) -> String 
pub struct CollabMatcher 
    pub fn new() -> Self 
    pub fn add_looking(&mut self, player_id: String, interests: Vec<String>) 
    pub fn remove_looking(&mut self, player_id: &str) 
    pub fn find_match(&self, player_id: &str) -> Option<String> 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**28 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
