#![forbid(unsafe_code)]

//! Exploration and discovery of unknown state space in balanced ternary {-1, 0, +1} systems.
//!
//! This crate provides structures for modeling how agents push into unknown territory:
//! the boundary between known and unknown (the frontier), agents that explore it,
//! and the rewards and risks associated with discovery.

use std::collections::{HashMap, HashSet};

/// A ternary value: Negative, Neutral, or Positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Neg = -1,
    Zero = 0,
    Pos = 1,
}

impl Ternary {
    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Ternary::Neg),
            0 => Some(Ternary::Zero),
            1 => Some(Ternary::Pos),
            _ => None,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }
}

/// Coordinates in a 2D state space, each axis is ternary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: Ternary,
    pub y: Ternary,
}

impl Coord {
    pub fn new(x: Ternary, y: Ternary) -> Self {
        Self { x, y }
    }

    /// All 9 possible coordinates in ternary 2D space.
    pub fn all() -> Vec<Coord> {
        let vals = [Ternary::Neg, Ternary::Zero, Ternary::Pos];
        let mut coords = Vec::with_capacity(9);
        for &x in &vals {
            for &y in &vals {
                coords.push(Coord::new(x, y));
            }
        }
        coords
    }
}

/// Whether a region is known (explored), unknown, or the boundary between them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegionStatus {
    Known,
    Frontier,
    Unknown,
}

/// A record of a single discovery.
#[derive(Debug, Clone)]
pub struct Discovery {
    pub coord: Coord,
    pub value: Ternary,
    pub discoverer: String,
    pub timestamp: u64,
}

/// Log of all discoveries made during exploration.
#[derive(Debug, Clone)]
pub struct DiscoveryLog {
    entries: Vec<Discovery>,
}

impl DiscoveryLog {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn record(&mut self, coord: Coord, value: Ternary, discoverer: &str, timestamp: u64) {
        self.entries.push(Discovery {
            coord,
            value,
            discoverer: discoverer.to_string(),
            timestamp,
        });
    }

    pub fn entries(&self) -> &[Discovery] {
        &self.entries
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Discoveries by a specific explorer.
    pub fn by_explorer(&self, name: &str) -> Vec<&Discovery> {
        self.entries.iter().filter(|d| d.discoverer == name).collect()
    }

    /// Check if a coordinate has been discovered.
    pub fn has_discovered(&self, coord: Coord) -> bool {
        self.entries.iter().any(|d| d.coord == coord)
    }

    /// Most recent discovery.
    pub fn latest(&self) -> Option<&Discovery> {
        self.entries.last()
    }
}

impl Default for DiscoveryLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Ternary risk assessment for frontier expansion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Safe,
    Caution,
    Dangerous,
}

/// Risk assessment result with reasoning.
#[derive(Debug, Clone)]
pub struct RiskAssessment {
    pub risk: RiskLevel,
    pub score: i8, // -1 (dangerous), 0 (caution), +1 (safe)
    pub reason: String,
}

impl RiskAssessment {
    pub fn assess(known_neighbors: usize, unknown_neighbors: usize, value: Ternary) -> Self {
        let score = match (known_neighbors, unknown_neighbors) {
            (n, _) if n >= 4 => 1,
            (n, u) if n >= 2 && u <= 2 => 0,
            _ => -1,
        };

        let risk = match score {
            1 => RiskLevel::Safe,
            0 => RiskLevel::Caution,
            -1 => RiskLevel::Dangerous,
            _ => RiskLevel::Dangerous,
        };

        let reason = match risk {
            RiskLevel::Safe => format!("{} known neighbors, well-charted territory", known_neighbors),
            RiskLevel::Caution => format!("Mixed region: {} known, {} unknown", known_neighbors, unknown_neighbors),
            RiskLevel::Dangerous => format!("Only {} known neighbors, high uncertainty", known_neighbors),
        };

        RiskAssessment {
            risk,
            score,
            reason,
            // value stored for future use
        }
    }

    /// Combined risk from ternary value: positive is safer, negative is riskier.
    pub fn from_ternary_value(value: Ternary) -> Self {
        match value {
            Ternary::Pos => RiskAssessment {
                risk: RiskLevel::Safe,
                score: 1,
                reason: "Positive signal indicates safe territory".to_string(),
            },
            Ternary::Zero => RiskAssessment {
                risk: RiskLevel::Caution,
                score: 0,
                reason: "Neutral signal, uncertain terrain".to_string(),
            },
            Ternary::Neg => RiskAssessment {
                risk: RiskLevel::Dangerous,
                score: -1,
                reason: "Negative signal indicates hostile territory".to_string(),
            },
        }
    }
}

/// The frontier: boundary between known and unknown regions.
#[derive(Debug, Clone)]
pub struct Frontier {
    known: HashSet<Coord>,
    frontier_coords: HashSet<Coord>,
    unknown: HashSet<Coord>,
}

impl Frontier {
    pub fn new() -> Self {
        let all: HashSet<Coord> = Coord::all().into_iter().collect();
        Self {
            known: HashSet::new(),
            frontier_coords: HashSet::new(),
            unknown: all,
        }
    }

    /// Claim a coordinate as explored.
    pub fn explore(&mut self, coord: Coord) {
        if self.unknown.remove(&coord) || self.frontier_coords.remove(&coord) {
            self.known.insert(coord);
            // Neighbors of the newly known coord become frontier
            for neighbor in self.neighbors(coord) {
                if self.unknown.contains(&neighbor) {
                    self.unknown.remove(&neighbor);
                    self.frontier_coords.insert(neighbor);
                }
            }
        }
    }

    /// Get direct neighbors (Manhattan distance 1) in ternary space, with wrap.
    fn neighbors(&self, coord: Coord) -> Vec<Coord> {
        let mut result = Vec::new();
        let vals = [Ternary::Neg, Ternary::Zero, Ternary::Pos];
        for &dx in &vals {
            for &dy in &vals {
                if dx == Ternary::Zero && dy == Ternary::Zero {
                    continue;
                }
                // Only Manhattan distance 1
                let mx = dx.to_i8().unsigned_abs() as usize;
                let my = dy.to_i8().unsigned_abs() as usize;
                if mx + my == 1 {
                    // Simple wrapping: add and mod 3, map back
                    let nx = ((coord.x.to_i8() + dx.to_i8()).rem_euclid(3) - 1) as i8;
                    let ny = ((coord.y.to_i8() + dy.to_i8()).rem_euclid(3) - 1) as i8;
                    if let (Some(x), Some(y)) = (Ternary::from_i8(nx), Ternary::from_i8(ny)) {
                        result.push(Coord::new(x, y));
                    }
                }
            }
        }
        result
    }

    pub fn known_count(&self) -> usize {
        self.known.len()
    }

    pub fn frontier_count(&self) -> usize {
        self.frontier_coords.len()
    }

    pub fn unknown_count(&self) -> usize {
        self.unknown.len()
    }

    pub fn frontier_coords(&self) -> &HashSet<Coord> {
        &self.frontier_coords
    }

    pub fn known_coords(&self) -> &HashSet<Coord> {
        &self.known
    }

    pub fn status(&self, coord: Coord) -> RegionStatus {
        if self.known.contains(&coord) {
            RegionStatus::Known
        } else if self.frontier_coords.contains(&coord) {
            RegionStatus::Frontier
        } else {
            RegionStatus::Unknown
        }
    }

    /// How complete is exploration? 0.0 to 1.0.
    pub fn completeness(&self) -> f64 {
        let total = 9.0;
        self.known.len() as f64 / total
    }

    /// Reset: return everything to unknown.
    pub fn reset(&mut self) {
        let all: HashSet<Coord> = Coord::all().into_iter().collect();
        self.known.clear();
        self.frontier_coords.clear();
        self.unknown = all;
    }
}

impl Default for Frontier {
    fn default() -> Self {
        Self::new()
    }
}

/// An explorer agent that pushes the frontier.
#[derive(Debug, Clone)]
pub struct Explorer {
    pub name: String,
    pub position: Option<Coord>,
    pub discoveries: usize,
    pub energy: u32,
}

impl Explorer {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            position: None,
            discoveries: 0,
            energy: 10,
        }
    }

    /// Move to a coordinate if there's energy.
    pub fn move_to(&mut self, coord: Coord) -> bool {
        if self.energy == 0 {
            return false;
        }
        self.position = Some(coord);
        self.energy -= 1;
        true
    }

    /// Make a discovery at the current position.
    pub fn discover(&mut self, value: Ternary, log: &mut DiscoveryLog, timestamp: u64) -> Option<Coord> {
        let coord = self.position?;
        log.record(coord, value, &self.name, timestamp);
        self.discoveries += 1;
        Some(coord)
    }

    /// Recharge energy.
    pub fn recharge(&mut self, amount: u32) {
        self.energy += amount;
    }

    pub fn is_exhausted(&self) -> bool {
        self.energy == 0
    }
}

/// Tracks which explorer first discovered each region (pioneer bonus).
#[derive(Debug, Clone)]
pub struct PioneerBonus {
    first_discoverer: HashMap<Coord, String>,
    bonuses: HashMap<String, u32>,
}

impl PioneerBonus {
    pub fn new() -> Self {
        Self {
            first_discoverer: HashMap::new(),
            bonuses: HashMap::new(),
        }
    }

    /// Record a discovery. Returns true if this explorer was first.
    pub fn record(&mut self, coord: Coord, explorer: &str) -> bool {
        if let std::collections::hash_map::Entry::Vacant(e) = self.first_discoverer.entry(coord) {
            e.insert(explorer.to_string());
            *self.bonuses.entry(explorer.to_string()).or_insert(0) += 1;
            return true;
        }
        false
    }

    /// Get the pioneer bonus count for an explorer.
    pub fn bonus(&self, explorer: &str) -> u32 {
        *self.bonuses.get(explorer).unwrap_or(&0)
    }

    /// Total distinct regions with pioneer bonuses.
    pub fn total_pioneered(&self) -> usize {
        self.first_discoverer.len()
    }

    /// Who discovered a coordinate first?
    pub fn discoverer_of(&self, coord: Coord) -> Option<&str> {
        self.first_discoverer.get(&coord).map(|s| s.as_str())
    }
}

impl Default for PioneerBonus {
    fn default() -> Self {
        Self::new()
    }
}

/// Claimed vs unclaimed territory map.
#[derive(Debug, Clone)]
pub struct TerritoryMap {
    claims: HashMap<Coord, String>,
}

impl TerritoryMap {
    pub fn new() -> Self {
        Self {
            claims: HashMap::new(),
        }
    }

    /// Claim a territory. Returns true if successful (unclaimed or own).
    pub fn claim(&mut self, coord: Coord, owner: &str) -> bool {
        match self.claims.get(&coord) {
            Some(existing) if existing != owner => false,
            _ => {
                self.claims.insert(coord, owner.to_string());
                true
            }
        }
    }

    /// Release a claim.
    pub fn release(&mut self, coord: Coord, owner: &str) -> bool {
        if self.claims.get(&coord).map(|o| o.as_str()) == Some(owner) {
            self.claims.remove(&coord);
            true
        } else {
            false
        }
    }

    /// Who owns this territory?
    pub fn owner(&self, coord: Coord) -> Option<&str> {
        self.claims.get(&coord).map(|s| s.as_str())
    }

    /// How many territories does an owner have?
    pub fn owned_by(&self, owner: &str) -> usize {
        self.claims.values().filter(|o| *o == owner).count()
    }

    /// Unclaimed coordinates.
    pub fn unclaimed(&self) -> Vec<Coord> {
        Coord::all().into_iter().filter(|c| !self.claims.contains_key(c)).collect()
    }

    /// Total claimed territories.
    pub fn total_claimed(&self) -> usize {
        self.claims.len()
    }
}

impl Default for TerritoryMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_from_i8() {
        assert_eq!(Ternary::from_i8(-1), Some(Ternary::Neg));
        assert_eq!(Ternary::from_i8(0), Some(Ternary::Zero));
        assert_eq!(Ternary::from_i8(1), Some(Ternary::Pos));
        assert_eq!(Ternary::from_i8(2), None);
        assert_eq!(Ternary::from_i8(-2), None);
    }

    #[test]
    fn test_ternary_to_i8() {
        assert_eq!(Ternary::Neg.to_i8(), -1);
        assert_eq!(Ternary::Zero.to_i8(), 0);
        assert_eq!(Ternary::Pos.to_i8(), 1);
    }

    #[test]
    fn test_coord_all_nine() {
        let all = Coord::all();
        assert_eq!(all.len(), 9);
        let unique: HashSet<_> = all.into_iter().collect();
        assert_eq!(unique.len(), 9);
    }

    #[test]
    fn test_frontier_starts_all_unknown() {
        let f = Frontier::new();
        assert_eq!(f.known_count(), 0);
        assert_eq!(f.frontier_count(), 0);
        assert_eq!(f.unknown_count(), 9);
        assert_eq!(f.completeness(), 0.0);
    }

    #[test]
    fn test_frontier_explore_one() {
        let mut f = Frontier::new();
        let center = Coord::new(Ternary::Zero, Ternary::Zero);
        f.explore(center);
        assert_eq!(f.known_count(), 1);
        assert!(f.frontier_count() > 0);
        assert_eq!(f.unknown_count(), 9 - 1 - f.frontier_count());
    }

    #[test]
    fn test_frontier_completeness_full() {
        let mut f = Frontier::new();
        for coord in Coord::all() {
            f.explore(coord);
        }
        assert_eq!(f.known_count(), 9);
        assert_eq!(f.completeness(), 1.0);
    }

    #[test]
    fn test_frontier_status() {
        let mut f = Frontier::new();
        let c = Coord::new(Ternary::Zero, Ternary::Zero);
        assert_eq!(f.status(c), RegionStatus::Unknown);
        f.explore(c);
        assert_eq!(f.status(c), RegionStatus::Known);
    }

    #[test]
    fn test_frontier_reset() {
        let mut f = Frontier::new();
        f.explore(Coord::new(Ternary::Zero, Ternary::Zero));
        f.reset();
        assert_eq!(f.known_count(), 0);
        assert_eq!(f.unknown_count(), 9);
    }

    #[test]
    fn test_discovery_log_record_and_count() {
        let mut log = DiscoveryLog::new();
        let c = Coord::new(Ternary::Pos, Ternary::Pos);
        log.record(c, Ternary::Pos, "alice", 100);
        assert_eq!(log.count(), 1);
        assert!(log.has_discovered(c));
    }

    #[test]
    fn test_discovery_log_by_explorer() {
        let mut log = DiscoveryLog::new();
        log.record(Coord::new(Ternary::Neg, Ternary::Zero), Ternary::Neg, "alice", 1);
        log.record(Coord::new(Ternary::Pos, Ternary::Zero), Ternary::Pos, "bob", 2);
        log.record(Coord::new(Ternary::Zero, Ternary::Zero), Ternary::Zero, "alice", 3);
        assert_eq!(log.by_explorer("alice").len(), 2);
        assert_eq!(log.by_explorer("bob").len(), 1);
    }

    #[test]
    fn test_discovery_log_latest() {
        let mut log = DiscoveryLog::new();
        assert!(log.latest().is_none());
        log.record(Coord::new(Ternary::Zero, Ternary::Zero), Ternary::Zero, "x", 1);
        log.record(Coord::new(Ternary::Pos, Ternary::Pos), Ternary::Pos, "x", 2);
        assert_eq!(log.latest().unwrap().timestamp, 2);
    }

    #[test]
    fn test_risk_assessment_safe() {
        let ra = RiskAssessment::assess(5, 1, Ternary::Pos);
        assert_eq!(ra.risk, RiskLevel::Safe);
        assert_eq!(ra.score, 1);
    }

    #[test]
    fn test_risk_assessment_dangerous() {
        let ra = RiskAssessment::assess(0, 5, Ternary::Neg);
        assert_eq!(ra.risk, RiskLevel::Dangerous);
        assert_eq!(ra.score, -1);
    }

    #[test]
    fn test_risk_from_ternary_value() {
        assert_eq!(RiskAssessment::from_ternary_value(Ternary::Pos).risk, RiskLevel::Safe);
        assert_eq!(RiskAssessment::from_ternary_value(Ternary::Zero).risk, RiskLevel::Caution);
        assert_eq!(RiskAssessment::from_ternary_value(Ternary::Neg).risk, RiskLevel::Dangerous);
    }

    #[test]
    fn test_explorer_move_and_energy() {
        let mut e = Explorer::new("scout");
        assert_eq!(e.energy, 10);
        assert!(e.move_to(Coord::new(Ternary::Zero, Ternary::Zero)));
        assert_eq!(e.energy, 9);
    }

    #[test]
    fn test_explorer_exhausted() {
        let mut e = Explorer::new("scout");
        e.energy = 0;
        assert!(e.is_exhausted());
        assert!(!e.move_to(Coord::new(Ternary::Zero, Ternary::Zero)));
    }

    #[test]
    fn test_explorer_discover() {
        let mut e = Explorer::new("scout");
        let mut log = DiscoveryLog::new();
        e.move_to(Coord::new(Ternary::Pos, Ternary::Neg));
        let coord = e.discover(Ternary::Pos, &mut log, 42);
        assert!(coord.is_some());
        assert_eq!(e.discoveries, 1);
        assert_eq!(log.count(), 1);
    }

    #[test]
    fn test_explorer_recharge() {
        let mut e = Explorer::new("scout");
        e.energy = 0;
        e.recharge(5);
        assert_eq!(e.energy, 5);
    }

    #[test]
    fn test_pioneer_bonus_first_wins() {
        let mut pb = PioneerBonus::new();
        let c = Coord::new(Ternary::Zero, Ternary::Zero);
        assert!(pb.record(c, "alice"));
        assert!(!pb.record(c, "bob"));
        assert_eq!(pb.bonus("alice"), 1);
        assert_eq!(pb.bonus("bob"), 0);
    }

    #[test]
    fn test_pioneer_bonus_discoverer_of() {
        let mut pb = PioneerBonus::new();
        let c = Coord::new(Ternary::Pos, Ternary::Pos);
        pb.record(c, "alice");
        assert_eq!(pb.discoverer_of(c), Some("alice"));
        let other = Coord::new(Ternary::Neg, Ternary::Neg);
        assert_eq!(pb.discoverer_of(other), None);
    }

    #[test]
    fn test_territory_claim_and_owner() {
        let mut tm = TerritoryMap::new();
        let c = Coord::new(Ternary::Zero, Ternary::Zero);
        assert!(tm.claim(c, "alice"));
        assert_eq!(tm.owner(c), Some("alice"));
        assert!(!tm.claim(c, "bob")); // already claimed
        assert!(tm.claim(c, "alice")); // own claim is ok
    }

    #[test]
    fn test_territory_release() {
        let mut tm = TerritoryMap::new();
        let c = Coord::new(Ternary::Zero, Ternary::Zero);
        tm.claim(c, "alice");
        assert!(tm.release(c, "alice"));
        assert_eq!(tm.owner(c), None);
    }

    #[test]
    fn test_territory_unclaimed() {
        let mut tm = TerritoryMap::new();
        assert_eq!(tm.unclaimed().len(), 9);
        tm.claim(Coord::new(Ternary::Zero, Ternary::Zero), "x");
        assert_eq!(tm.unclaimed().len(), 8);
    }

    #[test]
    fn test_territory_owned_by() {
        let mut tm = TerritoryMap::new();
        tm.claim(Coord::new(Ternary::Zero, Ternary::Zero), "alice");
        tm.claim(Coord::new(Ternary::Pos, Ternary::Pos), "alice");
        tm.claim(Coord::new(Ternary::Neg, Ternary::Neg), "bob");
        assert_eq!(tm.owned_by("alice"), 2);
        assert_eq!(tm.owned_by("bob"), 1);
    }
}
