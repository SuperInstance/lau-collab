//! # lau-collab
//!
//! Multiplayer collaboration layer for the Lau platform.
//! Kids play together, share worlds, and teach each other's agents.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

/// A player in the Lau ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub avatar_color: [f64; 3],
    pub world_id: String,
}

impl Player {
    pub fn new(id: &str, name: &str, avatar_color: [f64; 3], world_id: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            avatar_color,
            world_id: world_id.to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Permission
// ---------------------------------------------------------------------------

/// Access level for a collaborator in a shared world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Permission {
    Owner,
    Editor,
    #[default]
    Viewer,
}

// ---------------------------------------------------------------------------
// SharedWorld
// ---------------------------------------------------------------------------

/// A world that can be collaboratively edited by multiple players.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedWorld {
    pub world_id: String,
    pub owner: String,
    pub collaborators: HashMap<String, Permission>,
}

impl SharedWorld {
    pub fn new(world_id: &str, owner: &str) -> Self {
        let mut collaborators = HashMap::new();
        collaborators.insert(owner.to_string(), Permission::Owner);
        Self {
            world_id: world_id.to_string(),
            owner: owner.to_string(),
            collaborators,
        }
    }

    pub fn invite(&mut self, player_id: String, perm: Permission) {
        self.collaborators.insert(player_id, perm);
    }

    pub fn remove(&mut self, player_id: &str) -> bool {
        if player_id == self.owner {
            return false;
        }
        self.collaborators.remove(player_id).is_some()
    }

    pub fn is_member(&self, player_id: &str) -> bool {
        self.collaborators.contains_key(player_id)
    }

    pub fn permission(&self, player_id: &str) -> Permission {
        self.collaborators
            .get(player_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn can_build(&self, player_id: &str) -> bool {
        matches!(
            self.permission(player_id),
            Permission::Owner | Permission::Editor
        )
    }

    pub fn member_count(&self) -> usize {
        self.collaborators.len()
    }
}

// ---------------------------------------------------------------------------
// CollabEvent
// ---------------------------------------------------------------------------

/// An event that occurs during a collaboration session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollabEvent {
    PlayerJoined { player: String, world: String },
    PlayerLeft { player: String, world: String },
    CoBuild {
        players: Vec<String>,
        structure: String,
        voxels: usize,
    },
    AgentShared {
        from: String,
        to: String,
        agent_id: String,
    },
    WorldMerged {
        world_a: String,
        world_b: String,
        result: String,
    },
}

impl CollabEvent {
    /// Tag used for grouping events by type in `events_by_type`.
    fn tag(&self) -> &'static str {
        match self {
            CollabEvent::PlayerJoined { .. } => "PlayerJoined",
            CollabEvent::PlayerLeft { .. } => "PlayerLeft",
            CollabEvent::CoBuild { .. } => "CoBuild",
            CollabEvent::AgentShared { .. } => "AgentShared",
            CollabEvent::WorldMerged { .. } => "WorldMerged",
        }
    }

    /// Extract all player IDs referenced in this event.
    fn player_ids(&self) -> Vec<&str> {
        match self {
            CollabEvent::PlayerJoined { player, .. } => vec![player],
            CollabEvent::PlayerLeft { player, .. } => vec![player],
            CollabEvent::CoBuild { players, .. } => players.iter().map(String::as_str).collect(),
            CollabEvent::AgentShared { from, to, .. } => vec![from, to],
            CollabEvent::WorldMerged { .. } => vec![],
        }
    }
}

// ---------------------------------------------------------------------------
// SessionLog
// ---------------------------------------------------------------------------

/// A log of collaboration events within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLog {
    pub events: Vec<CollabEvent>,
    pub start_tick: u64,
    pub duration_ticks: u64,
}

impl SessionLog {
    pub fn new(start_tick: u64, duration_ticks: u64) -> Self {
        Self {
            events: Vec::new(),
            start_tick,
            duration_ticks,
        }
    }

    pub fn record(&mut self, event: CollabEvent) {
        self.events.push(event);
    }

    /// Unique player IDs that appear across all events.
    pub fn participants(&self) -> Vec<String> {
        let mut seen: Vec<String> = Vec::new();
        for ev in &self.events {
            for id in ev.player_ids() {
                if !seen.contains(&id.to_string()) {
                    seen.push(id.to_string());
                }
            }
        }
        seen
    }

    /// Count of each event type.
    pub fn events_by_type(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for ev in &self.events {
            *counts.entry(ev.tag().to_string()).or_insert(0) += 1;
        }
        counts
    }

    /// A fun narrative summary of the session.
    pub fn summary(&self) -> String {
        if self.events.is_empty() {
            return "An empty session — the world waited quietly, but nobody came.".to_string();
        }

        let mut parts: Vec<String> = Vec::new();
        let counts = self.events_by_type();

        let joins = counts.get("PlayerJoined").copied().unwrap_or(0);
        let leaves = counts.get("PlayerLeft").copied().unwrap_or(0);
        let builds = counts.get("CoBuild").copied().unwrap_or(0);
        let shares = counts.get("AgentShared").copied().unwrap_or(0);
        let merges = counts.get("WorldMerged").copied().unwrap_or(0);

        parts.push(format!(
            "🌟 Session report (tick {} – {}, {} ticks):",
            self.start_tick,
            self.start_tick + self.duration_ticks,
            self.duration_ticks
        ));

        if joins > 0 {
            parts.push(format!("🚪 {} player(s) joined the adventure.", joins));
        }
        if leaves > 0 {
            parts.push(format!("👋 {} player(s) headed home.", leaves));
        }
        if builds > 0 {
            parts.push(format!(
                "🏗️ {} co-build(s) brought ideas to life!",
                builds
            ));
        }
        if shares > 0 {
            parts.push(format!(
                "🤖 {} agent(s) were shared between friends.",
                shares
            ));
        }
        if merges > 0 {
            parts.push(format!(
                "🌌 {} world(s) were merged into something greater!",
                merges
            ));
        }

        let participants = self.participants();
        if !participants.is_empty() {
            parts.push(format!(
                "👥 {} unique player(s): {}",
                participants.len(),
                participants.join(", ")
            ));
        }

        parts.join("\n")
    }
}

// ---------------------------------------------------------------------------
// CollabMatcher
// ---------------------------------------------------------------------------

/// Matches kids who want to play together based on shared interests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollabMatcher {
    /// player_id -> set of interests
    looking: HashMap<String, Vec<String>>,
}

impl CollabMatcher {
    pub fn new() -> Self {
        Self {
            looking: HashMap::new(),
        }
    }

    pub fn add_looking(&mut self, player_id: String, interests: Vec<String>) {
        self.looking.insert(player_id, interests);
    }

    pub fn remove_looking(&mut self, player_id: &str) {
        self.looking.remove(player_id);
    }

    /// Find another player who shares at least one interest.
    pub fn find_match(&self, player_id: &str) -> Option<String> {
        let my_interests = self.looking.get(player_id)?;
        for (other_id, their_interests) in &self.looking {
            if other_id == player_id {
                continue;
            }
            if my_interests
                .iter()
                .any(|i| their_interests.contains(i))
            {
                return Some(other_id.clone());
            }
        }
        None
    }
}

impl Default for CollabMatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Player tests --

    #[test]
    fn player_new() {
        let p = Player::new("p1", "Ada", [1.0, 0.5, 0.0], "w1");
        assert_eq!(p.id, "p1");
        assert_eq!(p.name, "Ada");
        assert_eq!(p.avatar_color, [1.0, 0.5, 0.0]);
        assert_eq!(p.world_id, "w1");
    }

    #[test]
    fn player_equality() {
        let a = Player::new("p1", "Ada", [1.0, 0.0, 0.0], "w1");
        let b = Player::new("p1", "Ada", [1.0, 0.0, 0.0], "w1");
        assert_eq!(a, b);
    }

    // -- Permission tests --

    #[test]
    fn permission_default_is_viewer() {
        assert_eq!(Permission::default(), Permission::Viewer);
    }

    #[test]
    fn permission_equality() {
        assert_eq!(Permission::Owner, Permission::Owner);
        assert_ne!(Permission::Editor, Permission::Viewer);
    }

    // -- SharedWorld tests --

    #[test]
    fn world_new_owner_is_member() {
        let w = SharedWorld::new("w1", "alice");
        assert!(w.is_member("alice"));
        assert_eq!(w.permission("alice"), Permission::Owner);
        assert_eq!(w.member_count(), 1);
    }

    #[test]
    fn world_invite_editor() {
        let mut w = SharedWorld::new("w1", "alice");
        w.invite("bob".into(), Permission::Editor);
        assert!(w.is_member("bob"));
        assert_eq!(w.permission("bob"), Permission::Editor);
        assert_eq!(w.member_count(), 2);
    }

    #[test]
    fn world_invite_viewer() {
        let mut w = SharedWorld::new("w1", "alice");
        w.invite("carol".into(), Permission::Viewer);
        assert!(w.is_member("carol"));
        assert!(!w.can_build("carol"));
    }

    #[test]
    fn world_remove_collaborator() {
        let mut w = SharedWorld::new("w1", "alice");
        w.invite("bob".into(), Permission::Editor);
        assert!(w.remove("bob"));
        assert!(!w.is_member("bob"));
        assert_eq!(w.member_count(), 1);
    }

    #[test]
    fn world_cannot_remove_owner() {
        let mut w = SharedWorld::new("w1", "alice");
        assert!(!w.remove("alice"));
        assert!(w.is_member("alice"));
    }

    #[test]
    fn world_remove_nonexistent() {
        let mut w = SharedWorld::new("w1", "alice");
        assert!(!w.remove("ghost"));
    }

    #[test]
    fn world_can_build_owner_and_editor() {
        let mut w = SharedWorld::new("w1", "alice");
        w.invite("bob".into(), Permission::Editor);
        w.invite("carol".into(), Permission::Viewer);
        assert!(w.can_build("alice"));
        assert!(w.can_build("bob"));
        assert!(!w.can_build("carol"));
    }

    #[test]
    fn world_nonmember_is_viewer() {
        let w = SharedWorld::new("w1", "alice");
        assert_eq!(w.permission("nobody"), Permission::Viewer);
        assert!(!w.can_build("nobody"));
    }

    // -- SessionLog tests --

    #[test]
    fn log_record_and_count() {
        let mut log = SessionLog::new(0, 100);
        log.record(CollabEvent::PlayerJoined {
            player: "alice".into(),
            world: "w1".into(),
        });
        log.record(CollabEvent::PlayerJoined {
            player: "bob".into(),
            world: "w1".into(),
        });
        assert_eq!(log.events.len(), 2);
    }

    #[test]
    fn log_participants_dedupes() {
        let mut log = SessionLog::new(0, 100);
        log.record(CollabEvent::PlayerJoined {
            player: "alice".into(),
            world: "w1".into(),
        });
        log.record(CollabEvent::PlayerLeft {
            player: "alice".into(),
            world: "w1".into(),
        });
        log.record(CollabEvent::CoBuild {
            players: vec!["alice".into(), "bob".into()],
            structure: "castle".into(),
            voxels: 500,
        });
        let p = log.participants();
        assert_eq!(p.len(), 2);
        assert!(p.contains(&"alice".to_string()));
        assert!(p.contains(&"bob".to_string()));
    }

    #[test]
    fn log_events_by_type() {
        let mut log = SessionLog::new(0, 100);
        log.record(CollabEvent::PlayerJoined {
            player: "a".into(),
            world: "w".into(),
        });
        log.record(CollabEvent::PlayerJoined {
            player: "b".into(),
            world: "w".into(),
        });
        log.record(CollabEvent::CoBuild {
            players: vec!["a".into()],
            structure: "tower".into(),
            voxels: 10,
        });
        let counts = log.events_by_type();
        assert_eq!(counts.get("PlayerJoined"), Some(&2));
        assert_eq!(counts.get("CoBuild"), Some(&1));
    }

    #[test]
    fn log_empty_summary() {
        let log = SessionLog::new(0, 100);
        assert!(log.summary().contains("nobody came"));
    }

    #[test]
    fn log_summary_mentions_all_event_types() {
        let mut log = SessionLog::new(0, 60);
        log.record(CollabEvent::PlayerJoined {
            player: "alice".into(),
            world: "w1".into(),
        });
        log.record(CollabEvent::PlayerLeft {
            player: "bob".into(),
            world: "w1".into(),
        });
        log.record(CollabEvent::CoBuild {
            players: vec!["alice".into(), "carol".into()],
            structure: "rocket".into(),
            voxels: 999,
        });
        log.record(CollabEvent::AgentShared {
            from: "alice".into(),
            to: "carol".into(),
            agent_id: "ag1".into(),
        });
        log.record(CollabEvent::WorldMerged {
            world_a: "w1".into(),
            world_b: "w2".into(),
            result: "w3".into(),
        });
        let s = log.summary();
        assert!(s.contains("joined"));
        assert!(s.contains("headed home"));
        assert!(s.contains("co-build"));
        assert!(s.contains("agent"));
        assert!(s.contains("merged"));
    }

    #[test]
    fn log_agent_shared_participants() {
        let mut log = SessionLog::new(0, 100);
        log.record(CollabEvent::AgentShared {
            from: "alice".into(),
            to: "bob".into(),
            agent_id: "ag1".into(),
        });
        let p = log.participants();
        assert!(p.contains(&"alice".to_string()));
        assert!(p.contains(&"bob".to_string()));
    }

    #[test]
    fn log_world_merged_has_no_participants() {
        let mut log = SessionLog::new(0, 100);
        log.record(CollabEvent::WorldMerged {
            world_a: "w1".into(),
            world_b: "w2".into(),
            result: "w3".into(),
        });
        assert!(log.participants().is_empty());
    }

    // -- CollabMatcher tests --

    #[test]
    fn matcher_basic_match() {
        let mut m = CollabMatcher::new();
        m.add_looking("alice".into(), vec!["castles".into(), "space".into()]);
        m.add_looking("bob".into(), vec!["space".into(), "ocean".into()]);
        assert_eq!(m.find_match("alice"), Some("bob".into()));
        assert_eq!(m.find_match("bob"), Some("alice".into()));
    }

    #[test]
    fn matcher_no_match() {
        let mut m = CollabMatcher::new();
        m.add_looking("alice".into(), vec!["castles".into()]);
        m.add_looking("bob".into(), vec!["ocean".into()]);
        assert_eq!(m.find_match("alice"), None);
    }

    #[test]
    fn matcher_self_not_matched() {
        let mut m = CollabMatcher::new();
        m.add_looking("alice".into(), vec!["space".into()]);
        assert_eq!(m.find_match("alice"), None);
    }

    #[test]
    fn matcher_remove() {
        let mut m = CollabMatcher::new();
        m.add_looking("alice".into(), vec!["space".into()]);
        m.add_looking("bob".into(), vec!["space".into()]);
        m.remove_looking("bob");
        assert_eq!(m.find_match("alice"), None);
    }

    #[test]
    fn matcher_not_looking() {
        let m = CollabMatcher::new();
        assert_eq!(m.find_match("nobody"), None);
    }

    // -- Serde round-trip tests --

    #[test]
    fn serde_player() {
        let p = Player::new("p1", "Ada", [0.1, 0.2, 0.3], "w1");
        let json = serde_json::to_string(&p).unwrap();
        let back: Player = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn serde_collab_event() {
        let ev = CollabEvent::CoBuild {
            players: vec!["a".into(), "b".into()],
            structure: "fortress".into(),
            voxels: 42,
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: CollabEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }

    #[test]
    fn serde_shared_world() {
        let mut w = SharedWorld::new("w1", "alice");
        w.invite("bob".into(), Permission::Editor);
        let json = serde_json::to_string(&w).unwrap();
        let back: SharedWorld = serde_json::from_str(&json).unwrap();
        assert_eq!(back.world_id, "w1");
        assert_eq!(back.permission("bob"), Permission::Editor);
    }

    #[test]
    fn serde_session_log() {
        let mut log = SessionLog::new(10, 90);
        log.record(CollabEvent::PlayerJoined {
            player: "x".into(),
            world: "w".into(),
        });
        let json = serde_json::to_string(&log).unwrap();
        let back: SessionLog = serde_json::from_str(&json).unwrap();
        assert_eq!(back.events.len(), 1);
        assert_eq!(back.start_tick, 10);
    }
}
