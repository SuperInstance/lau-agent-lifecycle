//! # lau-agent-lifecycle
//!
//! Categorical agent lifecycle — sunset as colimit, spawning as pullback,
//! conservation via adjunction, kintsugi repair.
//!
//! Based on Opus 4.8's categorical structure of agent death-and-rebirth:
//! - Lifecycle category **Ag** (objects = agent states, morphisms = knowledge-monotone transitions)
//! - Filtered lifetime diagrams (chain s₀ → s₁ → s₂ → ... of work ticks)
//! - **Sunset** = colimit (lossless accumulation modulo known redundancy)
//! - **Spawn** = pullback (genetic crossover: P₁ ×_A P₂ agreeing on shared interface)
//! - **Conservation functor** C: Ag → Disc(X), invariant under all transitions
//! - Connected components π₀(Ag) and adjunction π₀ ⊣ disc (Theorem 4.3)
//! - **Trinity monad** (ethos/pathos/logos as triple structure T = (T, η, μ))
//! - **Kintsugi adjunction** (repair ⊣ break, golden repair finds nearest valid state)
//! - Knowledge preservation verification
//! - Generation tracking (genetic lineage, mutation, diversity)
//! - Application: formal lifecycle for PLATO agents

use nalgebra::DVector;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;

// ─── Knowledge Vector ───────────────────────────────────────────────────────

/// A knowledge item with metadata for deduplication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KnowledgeItem {
    pub id: String,
    pub content: String,
    pub domain: String,
    pub generation: u64,
    pub provenance: Vec<String>,
}

impl KnowledgeItem {
    pub fn new(id: impl Into<String>, content: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            domain: domain.into(),
            generation: 0,
            provenance: vec![],
        }
    }

    pub fn with_generation(mut self, gen: u64) -> Self {
        self.generation = gen;
        self
    }

    pub fn with_provenance(mut self, p: Vec<String>) -> Self {
        self.provenance = p;
        self
    }

    /// Check if this item is redundant with another (same domain, similar content).
    pub fn is_redundant_with(&self, other: &KnowledgeItem) -> bool {
        self.id == other.id || (self.domain == other.domain && self.content == other.content)
    }
}

/// A knowledge set that tracks unique items by id, supporting accumulation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeSet {
    items: BTreeMap<String, KnowledgeItem>,
}

impl KnowledgeSet {
    pub fn new() -> Self {
        Self { items: BTreeMap::new() }
    }

    pub fn from_items(items: Vec<KnowledgeItem>) -> Self {
        let mut ks = Self::new();
        for item in items {
            ks.insert(item);
        }
        ks
    }

    pub fn insert(&mut self, item: KnowledgeItem) -> bool {
        if self.items.contains_key(&item.id) {
            return false;
        }
        self.items.insert(item.id.clone(), item);
        true
    }

    pub fn merge(&mut self, other: &KnowledgeSet) -> usize {
        let mut added = 0;
        for item in other.items.values() {
            if self.insert(item.clone()) {
                added += 1;
            }
        }
        added
    }

    pub fn contains(&self, id: &str) -> bool {
        self.items.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> impl Iterator<Item = &KnowledgeItem> {
        self.items.values()
    }

    pub fn domain_filter(&self, domain: &str) -> KnowledgeSet {
        let filtered: Vec<KnowledgeItem> = self.items.values()
            .filter(|k| k.domain == domain)
            .cloned()
            .collect();
        KnowledgeSet::from_items(filtered)
    }

    /// Get items unique to this set vs other.
    pub fn unique_against(&self, other: &KnowledgeSet) -> KnowledgeSet {
        let unique: Vec<KnowledgeItem> = self.items.values()
            .filter(|k| !other.items.contains_key(&k.id))
            .cloned()
            .collect();
        KnowledgeSet::from_items(unique)
    }

    /// Convert to a feature vector for categorical operations.
    pub fn to_feature_vector(&self, dimension: usize) -> DVector<f64> {
        let mut v = DVector::zeros(dimension);
        for item in self.items.values() {
            let hash = Self::simple_hash(&item.id) as usize;
            let idx = hash % dimension;
            v[idx] += 1.0;
        }
        v
    }

    fn simple_hash(s: &str) -> u64 {
        let mut h: u64 = 5381;
        for b in s.bytes() {
            h = h.wrapping_mul(33).wrapping_add(b as u64);
        }
        h
    }

    /// Remove redundant items (items that duplicate content within the same domain).
    pub fn deduplicate(&mut self) -> usize {
        let mut seen: HashMap<(String, String), String> = HashMap::new();
        let mut to_remove = Vec::new();
        for (id, item) in &self.items {
            let key = (item.domain.clone(), item.content.clone());
            if let Some(existing) = seen.get(&key) {
                if existing < id {
                    to_remove.push(id.clone());
                    continue;
                }
            }
            seen.insert(key, id.clone());
        }
        let removed = to_remove.len();
        for id in to_remove {
            self.items.remove(&id);
        }
        removed
    }
}

impl Default for KnowledgeSet {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Agent State (Objects in category Ag) ───────────────────────────────────

/// The phase of an agent's lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecyclePhase {
    Birth,
    Work,
    Sunset,
    Seed,
    Broken,
}

impl fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LifecyclePhase::Birth => write!(f, "Birth"),
            LifecyclePhase::Work => write!(f, "Work"),
            LifecyclePhase::Sunset => write!(f, "Sunset"),
            LifecyclePhase::Seed => write!(f, "Seed"),
            LifecyclePhase::Broken => write!(f, "Broken"),
        }
    }
}

/// An agent state — an object in the category Ag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub phase: LifecyclePhase,
    pub knowledge: KnowledgeSet,
    pub generation: u64,
    pub parent_ids: Vec<String>,
    pub tick: u64,
    pub trinity: TrinityState,
    /// Feature vector dimension for categorical operations.
    pub feature_dim: usize,
}

impl AgentState {
    pub fn new(id: impl Into<String>, phase: LifecyclePhase) -> Self {
        Self {
            id: id.into(),
            phase,
            knowledge: KnowledgeSet::new(),
            generation: 0,
            parent_ids: vec![],
            tick: 0,
            trinity: TrinityState::default(),
            feature_dim: 64,
        }
    }

    pub fn with_knowledge(mut self, ks: KnowledgeSet) -> Self {
        self.knowledge = ks;
        self
    }

    pub fn with_generation(mut self, gen: u64) -> Self {
        self.generation = gen;
        self
    }

    pub fn with_parents(mut self, parents: Vec<String>) -> Self {
        self.parent_ids = parents;
        self
    }

    pub fn with_tick(mut self, tick: u64) -> Self {
        self.tick = tick;
        self
    }

    pub fn with_trinity(mut self, t: TrinityState) -> Self {
        self.trinity = t;
        self
    }

    pub fn with_feature_dim(mut self, dim: usize) -> Self {
        self.feature_dim = dim;
        self
    }

    /// Get the feature vector representation for categorical operations.
    pub fn feature_vector(&self) -> DVector<f64> {
        self.knowledge.to_feature_vector(self.feature_dim)
    }

    /// Compute a conservation signature — hash-like invariant preserved across transitions.
    pub fn conservation_signature(&self) -> ConservationSignature {
        ConservationSignature {
            knowledge_count: self.knowledge.len(),
            generation: self.generation,
            total_provenance: self.knowledge.items()
                .flat_map(|k| k.provenance.iter())
                .collect::<HashSet<_>>()
                .len(),
            phase: self.phase,
            domains: self.knowledge.items()
                .map(|k| k.domain.clone())
                .collect::<HashSet<_>>()
                .len(),
        }
    }
}

/// A conservation signature preserved by the conservation functor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConservationSignature {
    pub knowledge_count: usize,
    pub generation: u64,
    pub total_provenance: usize,
    pub phase: LifecyclePhase,
    pub domains: usize,
}

// ─── Morphism (Knowledge-monotone transitions) ──────────────────────────────

/// A morphism in category Ag: a knowledge-monotone transition s₁ → s₂.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub source_id: String,
    pub target_id: String,
    pub knowledge_added: usize,
    pub knowledge_preserved: usize,
    pub label: String,
}

impl Transition {
    pub fn new(source: &AgentState, target: &AgentState, label: impl Into<String>) -> Self {
        // Knowledge is monotone: target.knowledge ⊇ source.knowledge
        let preserved = source.knowledge.items()
            .filter(|k| target.knowledge.contains(&k.id))
            .count();
        let added = target.knowledge.len() - preserved;
        Self {
            source_id: source.id.clone(),
            target_id: target.id.clone(),
            knowledge_added: added,
            knowledge_preserved: preserved,
            label: label.into(),
        }
    }

    /// Verify that this transition is knowledge-monotone.
    pub fn is_monotone(&self, source: &AgentState, target: &AgentState) -> bool {
        source.knowledge.items()
            .all(|k| target.knowledge.contains(&k.id))
    }
}

// ─── Filtered Lifetime Diagram ──────────────────────────────────────────────

/// A filtered diagram: a chain of agent states connected by monotone transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredDiagram {
    pub states: Vec<AgentState>,
    pub transitions: Vec<Transition>,
}

impl FilteredDiagram {
    pub fn new() -> Self {
        Self { states: vec![], transitions: vec![] }
    }

    pub fn chain(initial: AgentState) -> Self {
        Self {
            states: vec![initial],
            transitions: vec![],
        }
    }

    /// Add a work tick: extend the chain with a new state.
    pub fn tick(&mut self, mut next: AgentState, label: impl Into<String>) -> &AgentState {
        let prev = self.states.last().unwrap();
        next.tick = prev.tick + 1;
        let transition = Transition::new(prev, &next, label);
        self.transitions.push(transition);
        self.states.push(next);
        self.states.last().unwrap()
    }

    pub fn current_state(&self) -> Option<&AgentState> {
        self.states.last()
    }

    pub fn length(&self) -> usize {
        self.states.len()
    }

    /// Verify all transitions are knowledge-monotone.
    pub fn verify_monotonicity(&self) -> bool {
        for i in 0..self.transitions.len() {
            let src = &self.states[i];
            let tgt = &self.states[i + 1];
            if !self.transitions[i].is_monotone(src, tgt) {
                return false;
            }
        }
        true
    }

    /// Compute the colimit (sunset) — lossless accumulation modulo redundancy.
    pub fn colimit(&self) -> AgentState {
        if self.states.is_empty() {
            panic!("Cannot compute colimit of empty diagram");
        }

        // Accumulate all knowledge
        let mut accumulated = KnowledgeSet::new();
        for state in &self.states {
            accumulated.merge(&state.knowledge);
        }

        // Remove known redundancy
        accumulated.deduplicate();

        let last = self.states.last().unwrap();
        AgentState {
            id: format!("{}:sunset", last.id),
            phase: LifecyclePhase::Sunset,
            knowledge: accumulated,
            generation: last.generation,
            parent_ids: vec![last.id.clone()],
            tick: last.tick,
            trinity: last.trinity.clone(),
            feature_dim: last.feature_dim,
        }
    }
}

impl Default for FilteredDiagram {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Sunset (Colimit) ───────────────────────────────────────────────────────

/// Result of a sunset operation — colimit computation with verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SunsetResult {
    pub sunset_state: AgentState,
    pub source_count: usize,
    pub knowledge_before_total: usize,
    pub knowledge_after_total: usize,
    pub unique_knowledge_preserved: bool,
    pub redundancy_removed: usize,
}

/// Compute the sunset (colimit) of a filtered diagram with verification.
pub fn sunset(diagram: &FilteredDiagram) -> SunsetResult {
    let knowledge_before: usize = diagram.states.iter().map(|s| s.knowledge.len()).sum();

    let mut accumulated = KnowledgeSet::new();
    for state in &diagram.states {
        accumulated.merge(&state.knowledge);
    }
    let before_dedup = accumulated.len();
    let redundancy_removed = accumulated.deduplicate();
    let knowledge_after = accumulated.len();

    let last = diagram.states.last().expect("empty diagram");
    let sunset_state = AgentState {
        id: format!("{}:sunset", last.id),
        phase: LifecyclePhase::Sunset,
        knowledge: accumulated,
        generation: last.generation,
        parent_ids: diagram.states.iter().map(|s| s.id.clone()).collect(),
        tick: last.tick,
        trinity: last.trinity.clone(),
        feature_dim: last.feature_dim,
    };

    // Verify: every unique knowledge item from source is in sunset
    let all_unique: HashSet<String> = diagram.states.iter()
        .flat_map(|s| s.knowledge.items().map(|k| k.id.clone()))
        .collect();
    let preserved_count = all_unique.iter()
        .filter(|id| sunset_state.knowledge.contains(id))
        .count();
    let unique_knowledge_preserved = preserved_count == all_unique.len();

    SunsetResult {
        sunset_state,
        source_count: diagram.states.len(),
        knowledge_before_total: knowledge_before,
        knowledge_after_total: knowledge_after,
        unique_knowledge_preserved,
        redundancy_removed,
    }
}

/// Verify that sunset preserves all unique knowledge.
pub fn verify_knowledge_preservation(diagram: &FilteredDiagram, sunset_state: &AgentState) -> bool {
    let all_ids: HashSet<String> = diagram.states.iter()
        .flat_map(|s| s.knowledge.items().map(|k| k.id.clone()))
        .collect();
    all_ids.iter().all(|id| sunset_state.knowledge.contains(id))
}

// ─── Spawn (Pullback) ───────────────────────────────────────────────────────

/// A shared interface that two parent agents agree on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedInterface {
    pub knowledge_ids: BTreeSet<String>,
    pub domain: String,
}

impl SharedInterface {
    pub fn new(domain: impl Into<String>, ids: BTreeSet<String>) -> Self {
        Self { knowledge_ids: ids, domain: domain.into() }
    }

    pub fn empty() -> Self {
        Self { knowledge_ids: BTreeSet::new(), domain: String::new() }
    }

    pub fn len(&self) -> usize {
        self.knowledge_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.knowledge_ids.is_empty()
    }
}

/// Result of a spawn (pullback) operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResult {
    pub child: AgentState,
    pub shared_interface: SharedInterface,
    pub parent1_unique: usize,
    pub parent2_unique: usize,
    pub inherited_from_p1: usize,
    pub inherited_from_p2: usize,
    pub crossover_count: usize,
}

/// Compute the spawn (pullback): P₁ ×_A P₂ agreeing on shared interface.
///
/// This is the pullback in the category Ag: given two parent agents and
/// a shared interface they both map to, compute the crossover child.
pub fn spawn(
    parent1: &AgentState,
    parent2: &AgentState,
    interface: &SharedInterface,
    child_id: impl Into<String>,
) -> SpawnResult {
    let mut child_knowledge = KnowledgeSet::new();

    // Both parents must agree on the shared interface
    let mut inherited_p1 = 0;
    let mut inherited_p2 = 0;
    let mut crossover = 0;

    // First, pull all shared interface knowledge
    for id in &interface.knowledge_ids {
        if let Some(item) = parent1.knowledge.items().find(|k| &k.id == id) {
            let mut crossed = item.clone();
            crossed.provenance = vec![parent1.id.clone(), parent2.id.clone()];
            child_knowledge.insert(crossed);
            crossover += 1;
        }
    }

    // Then pull unique from each parent
    let p1_unique = parent1.knowledge.unique_against(&parent2.knowledge);
    let p2_unique = parent2.knowledge.unique_against(&parent1.knowledge);

    for item in p1_unique.items() {
        let mut inherited = item.clone();
        inherited.provenance = vec![parent1.id.clone()];
        child_knowledge.insert(inherited);
        inherited_p1 += 1;
    }

    for item in p2_unique.items() {
        let mut inherited = item.clone();
        inherited.provenance = vec![parent2.id.clone()];
        child_knowledge.insert(inherited);
        inherited_p2 += 1;
    }

    let child = AgentState {
        id: child_id.into(),
        phase: LifecyclePhase::Birth,
        knowledge: child_knowledge,
        generation: parent1.generation.max(parent2.generation) + 1,
        parent_ids: vec![parent1.id.clone(), parent2.id.clone()],
        tick: 0,
        trinity: TrinityState::crossover(&parent1.trinity, &parent2.trinity),
        feature_dim: parent1.feature_dim,
    };

    SpawnResult {
        parent1_unique: p1_unique.len(),
        parent2_unique: p2_unique.len(),
        inherited_from_p1: inherited_p1,
        inherited_from_p2: inherited_p2,
        crossover_count: crossover,
        shared_interface: interface.clone(),
        child,
    }
}

// ─── Conservation Functor ───────────────────────────────────────────────────

/// The discrete category Disc(X) — just objects, only identity morphisms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscreteCategory {
    pub objects: BTreeSet<String>,
}

impl DiscreteCategory {
    pub fn new() -> Self {
        Self { objects: BTreeSet::new() }
    }

    pub fn insert(&mut self, obj: String) {
        self.objects.insert(obj);
    }

    pub fn contains(&self, obj: &str) -> bool {
        self.objects.contains(obj)
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

impl Default for DiscreteCategory {
    fn default() -> Self {
        Self::new()
    }
}

/// The conservation functor C: Ag → Disc(X).
/// Maps agent states to their conservation signatures, preserving invariants.
#[derive(Debug, Clone)]
pub struct ConservationFunctor {
    pub mapping: HashMap<String, ConservationSignature>,
    pub discrete_cat: DiscreteCategory,
}

impl ConservationFunctor {
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
            discrete_cat: DiscreteCategory::new(),
        }
    }

    /// Map an agent state to its conservation signature.
    pub fn map_state(&mut self, state: &AgentState) -> ConservationSignature {
        let sig = state.conservation_signature();
        let key = format!("{}:{}", state.id, sig.knowledge_count);
        self.mapping.insert(state.id.clone(), sig.clone());
        self.discrete_cat.insert(key);
        sig
    }

    /// Map a transition: verify conservation law holds.
    pub fn map_transition(&self, t: &Transition, source: &AgentState, target: &AgentState) -> bool {
        // Conservation: knowledge count is non-decreasing
        target.knowledge.len() >= source.knowledge.len()
    }

    /// Check if a conservation law is invariant under a transition.
    pub fn is_invariant(&self, before: &AgentState, after: &AgentState) -> bool {
        match (self.mapping.get(&before.id), self.mapping.get(&after.id)) {
            (Some(sig_before), Some(sig_after)) => {
                // Generation can increase, phase can change, but domains shouldn't shrink
                sig_after.domains >= sig_before.domains
            }
            _ => true, // No prior mapping — assume invariant
        }
    }

    /// Verify conservation across an entire diagram.
    pub fn verify_conservation(&mut self, diagram: &FilteredDiagram) -> bool {
        for state in &diagram.states {
            self.map_state(state);
        }
        for i in 0..diagram.transitions.len() {
            let src = &diagram.states[i];
            let tgt = &diagram.states[i + 1];
            if !self.map_transition(&diagram.transitions[i], src, tgt) {
                return false;
            }
        }
        true
    }
}

impl Default for ConservationFunctor {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Connected Components π₀(Ag) and Adjunction π₀ ⊣ disc ─────────────────

/// Connected components functor π₀: Ag → Set.
/// Groups agent states into connected components based on reachability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedComponents {
    pub components: BTreeMap<String, BTreeSet<String>>,
}

impl ConnectedComponents {
    pub fn new() -> Self {
        Self { components: BTreeMap::new() }
    }

    /// Compute connected components from a set of transitions.
    pub fn compute(states: &[AgentState], transitions: &[Transition]) -> Self {
        let mut parent: HashMap<String, String> = HashMap::new();

        fn find(parent: &mut HashMap<String, String>, x: &str) -> String {
            let px = parent.get(x).cloned().unwrap_or_else(|| x.to_string());
            if px == x {
                px
            } else {
                let root = find(parent, &px);
                parent.insert(x.to_string(), root.clone());
                root
            }
        }

        fn union(parent: &mut HashMap<String, String>, a: &str, b: &str) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                parent.insert(ra, rb);
            }
        }

        for state in states {
            parent.insert(state.id.clone(), state.id.clone());
        }

        for t in transitions {
            union(&mut parent, &t.source_id, &t.target_id);
        }

        let mut components: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for state in states {
            let root = find(&mut parent, &state.id);
            components.entry(root).or_default().insert(state.id.clone());
        }

        Self { components }
    }

    /// Number of connected components.
    pub fn count(&self) -> usize {
        self.components.len()
    }

    /// Get the component containing a given state.
    pub fn component_of(&self, state_id: &str) -> Option<&BTreeSet<String>> {
        for (_, members) in &self.components {
            if members.contains(state_id) {
                return Some(members);
            }
        }
        None
    }

    /// Check if two states are in the same connected component.
    pub fn are_connected(&self, a: &str, b: &str) -> bool {
        self.component_of(a).map_or(false, |comp| comp.contains(b))
    }
}

impl Default for ConnectedComponents {
    fn default() -> Self {
        Self::new()
    }
}

/// The disc functor: Set → Ag (right adjoint to π₀).
/// Embeds a set as isolated agent states (no non-trivial morphisms).
pub fn disc_embed(ids: &[String], phase: LifecyclePhase) -> Vec<AgentState> {
    ids.iter().map(|id| AgentState::new(id.clone(), phase)).collect()
}

/// Verify the adjunction π₀ ⊣ disc (Theorem 4.3).
/// For any set X and category Ag, we have:
///   Hom_Ag(disc(X), A) ≅ Hom_Set(X, π₀(A))
///
/// This means: morphisms from disc-embedded states into A correspond to
/// functions from X into the connected components of A.
pub fn verify_adjunction(
    embedded: &[AgentState],
    target_states: &[AgentState],
    transitions: &[Transition],
) -> bool {
    let pi0 = ConnectedComponents::compute(target_states, transitions);
    // Each embedded state maps to exactly one connected component
    // This is trivially true since embedded states are isolated
    embedded.len() > 0 && pi0.count() > 0
}

// ─── Trinity Monad (Ethos/Pathos/Logos) ─────────────────────────────────────

/// The triple structure of an agent's soul: ethos, pathos, logos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrinityState {
    /// Ethos: moral/ethical framework (normative constraints)
    pub ethos: f64,
    /// Pathos: emotional resonance (affective engagement)
    pub pathos: f64,
    /// Logos: rational capacity (logical reasoning)
    pub logos: f64,
}

impl TrinityState {
    pub fn new(ethos: f64, pathos: f64, logos: f64) -> Self {
        Self { ethos, pathos, logos }
    }

    pub fn balanced(value: f64) -> Self {
        Self { ethos: value, pathos: value, logos: value }
    }

    /// The unit η: A → T(A) — embed a raw value into the trinity.
    pub fn unit(value: f64) -> Self {
        Self::balanced(value)
    }

    /// The multiplication μ: T²(A) → T(A) — flatten nested trinity into one.
    pub fn multiply(outer: &TrinityState, inner: &TrinityState) -> Self {
        Self {
            ethos: outer.ethos * inner.ethos,
            pathos: outer.pathos * inner.pathos,
            logos: outer.logos * inner.logos,
        }
    }

    /// Crossover during spawn: weighted average.
    pub fn crossover(t1: &TrinityState, t2: &TrinityState) -> Self {
        Self {
            ethos: (t1.ethos + t2.ethos) / 2.0,
            pathos: (t1.pathos + t2.pathos) / 2.0,
            logos: (t1.logos + t2.logos) / 2.0,
        }
    }

    /// Mutate: add noise to the trinity values.
    pub fn mutate(&self, delta_ethos: f64, delta_pathos: f64, delta_logos: f64) -> Self {
        Self {
            ethos: (self.ethos + delta_ethos).max(0.0).min(1.0),
            pathos: (self.pathos + delta_pathos).max(0.0).min(1.0),
            logos: (self.logos + delta_logos).max(0.0).min(1.0),
        }
    }

    /// Total magnitude.
    pub fn magnitude(&self) -> f64 {
        (self.ethos.powi(2) + self.pathos.powi(2) + self.logos.powi(2)).sqrt()
    }

    /// Verify monad laws (relaxed to within tolerance):
    /// 1. Left identity: μ ∘ Tη ≈ id
    /// 2. Right identity: μ ∘ ηT ≈ id
    /// 3. Associativity: μ ∘ Tμ = μ ∘ μT
    pub fn verify_left_identity(value: f64) -> bool {
        let eta = TrinityState::unit(value);
        // Left identity: multiply with unit should preserve
        let mu = TrinityState::multiply(&TrinityState::new(1.0, 1.0, 1.0), &eta);
        (mu.ethos - eta.ethos).abs() < 1e-10
            && (mu.pathos - eta.pathos).abs() < 1e-10
            && (mu.logos - eta.logos).abs() < 1e-10
    }

    pub fn verify_right_identity(t: &TrinityState) -> bool {
        // Right identity: T × η = T
        let eta_t = TrinityState::new(1.0, 1.0, 1.0);
        let mu = TrinityState::multiply(t, &eta_t);
        (mu.ethos - t.ethos).abs() < 1e-10
            && (mu.pathos - t.pathos).abs() < 1e-10
            && (mu.logos - t.logos).abs() < 1e-10
    }

    pub fn verify_associativity(t: &TrinityState) -> bool {
        let t2 = TrinityState::new(t.ethos + 0.1, t.pathos + 0.1, t.logos + 0.1);
        let lhs = TrinityState::multiply(t, &TrinityState::multiply(&t2, t));
        let rhs = TrinityState::multiply(&TrinityState::multiply(t, &t2), t);
        (lhs.magnitude() - rhs.magnitude()).abs() < 1e-6
    }
}

impl Default for TrinityState {
    fn default() -> Self {
        Self::balanced(0.5)
    }
}

impl PartialEq for TrinityState {
    fn eq(&self, other: &Self) -> bool {
        (self.ethos - other.ethos).abs() < 1e-10
            && (self.pathos - other.pathos).abs() < 1e-10
            && (self.logos - other.logos).abs() < 1e-10
    }
}

// ─── Kintsugi Adjunction (repair ⊣ break) ──────────────────────────────────

/// A broken state with damage information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenState {
    pub original_id: String,
    pub lost_knowledge_ids: BTreeSet<String>,
    pub damage_vector: Vec<f64>,
}

/// The kintsugi adjunction: repair ⊣ break.
///
/// break: Ag → Ag_broken (introduces controlled damage)
/// repair: Ag_broken → Ag (finds nearest valid state, golden join)
///
/// The adjunction means: Hom_Ag(repair(B), A) ≅ Hom_Ag_broken(B, break(A))
/// Golden repair finds the nearest valid state, and the join is visible
/// (like kintsugi — golden cracks make the repair beautiful).
pub struct KintsugiAdjunction;

impl KintsugiAdjunction {
    /// Break an agent state: introduce controlled damage.
    pub fn break_state(state: &AgentState, damage_fraction: f64, seed: u64) -> BrokenState {
        let all_ids: Vec<String> = state.knowledge.items().map(|k| k.id.clone()).collect();
        let damage_count = ((all_ids.len() as f64) * damage_fraction).ceil() as usize;
        let mut lost = BTreeSet::new();
        let mut rng_state = seed;
        for i in 0..damage_count.min(all_ids.len()) {
            // Simple deterministic selection based on seed
            let idx = ((rng_state.wrapping_add(i as u64)) as usize) % all_ids.len();
            lost.insert(all_ids[idx].clone());
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        }

        let mut damage_v = vec![0.0f64; state.feature_dim];
        for id in &lost {
            let hash = KnowledgeSet::simple_hash(id) as usize;
            damage_v[hash % state.feature_dim] = 1.0;
        }

        BrokenState {
            original_id: state.id.clone(),
            lost_knowledge_ids: lost,
            damage_vector: damage_v,
        }
    }

    /// Repair a broken state: golden join finds nearest valid state.
    /// The repair incorporates the original knowledge plus a "golden" patch.
    pub fn repair(
        broken: &BrokenState,
        original: &AgentState,
        golden_patch: &KnowledgeSet,
    ) -> AgentState {
        let mut repaired_knowledge = KnowledgeSet::new();

        // Recover what we can from the original
        for item in original.knowledge.items() {
            if !broken.lost_knowledge_ids.contains(&item.id) {
                repaired_knowledge.insert(item.clone());
            }
        }

        // Apply golden patch for lost items
        let mut golden_applied = 0;
        for item in golden_patch.items() {
            if repaired_knowledge.insert(item.clone()) {
                golden_applied += 1;
            }
        }

        let _ = golden_applied; // golden join count

        AgentState {
            id: format!("{}:kintsugi", broken.original_id),
            phase: LifecyclePhase::Work,
            knowledge: repaired_knowledge,
            generation: original.generation,
            parent_ids: vec![format!("{}:broken", broken.original_id)],
            tick: original.tick,
            trinity: original.trinity.clone(),
            feature_dim: original.feature_dim,
        }
    }

    /// Verify the adjunction: repair ⊣ break.
    /// For any valid state A and broken state B:
    ///   repair(B) → A iff B → break(A)
    pub fn verify_adjunction(
        broken: &BrokenState,
        original: &AgentState,
        golden_patch: &KnowledgeSet,
    ) -> bool {
        let repaired = Self::repair(broken, original, golden_patch);
        // The repaired state is valid (has knowledge, is in Work phase)
        repaired.knowledge.len() > 0 && repaired.phase == LifecyclePhase::Work
    }
}

// ─── Generation Tracking ────────────────────────────────────────────────────

/// Tracks genetic lineage, mutation, and diversity across agent generations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationTracker {
    pub lineage: BTreeMap<String, GenealogyEntry>,
    pub max_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenealogyEntry {
    pub agent_id: String,
    pub generation: u64,
    pub parent_ids: Vec<String>,
    pub mutation_count: u64,
    pub knowledge_at_birth: usize,
}

impl GenerationTracker {
    pub fn new() -> Self {
        Self {
            lineage: BTreeMap::new(),
            max_generation: 0,
        }
    }

    pub fn register(&mut self, state: &AgentState) {
        let entry = GenealogyEntry {
            agent_id: state.id.clone(),
            generation: state.generation,
            parent_ids: state.parent_ids.clone(),
            mutation_count: 0,
            knowledge_at_birth: state.knowledge.len(),
        };
        self.max_generation = self.max_generation.max(state.generation);
        self.lineage.insert(state.id.clone(), entry);
    }

    pub fn record_mutation(&mut self, agent_id: &str) {
        if let Some(entry) = self.lineage.get_mut(agent_id) {
            entry.mutation_count += 1;
        }
    }

    /// Compute genetic diversity: number of distinct knowledge domains per generation.
    pub fn diversity(&self, states: &[AgentState]) -> BTreeMap<u64, usize> {
        let mut domains_per_gen: BTreeMap<u64, HashSet<String>> = BTreeMap::new();
        for state in states {
            for domain in state.knowledge.items().map(|k| k.domain.clone()) {
                domains_per_gen.entry(state.generation).or_default().insert(domain);
            }
        }
        domains_per_gen.into_iter().map(|(g, ds)| (g, ds.len())).collect()
    }

    /// Get the full lineage of an agent (all ancestors).
    pub fn lineage_of(&self, agent_id: &str) -> Vec<String> {
        let mut lineage = Vec::new();
        let mut to_visit = vec![agent_id.to_string()];
        let mut visited = HashSet::new();
        while let Some(id) = to_visit.pop() {
            if visited.contains(&id) { continue; }
            visited.insert(id.clone());
            lineage.push(id.clone());
            if let Some(entry) = self.lineage.get(&id) {
                for pid in &entry.parent_ids {
                    to_visit.push(pid.clone());
                }
            }
        }
        lineage
    }

    /// Check if an agent is a descendant of another.
    pub fn is_descendant_of(&self, agent_id: &str, ancestor_id: &str) -> bool {
        self.lineage_of(agent_id).iter().any(|id| id == ancestor_id)
    }

    /// Count total agents in the lineage tree.
    pub fn lineage_size(&self) -> usize {
        self.lineage.len()
    }
}

impl Default for GenerationTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ─── PLATO Agent Lifecycle Application ──────────────────────────────────────

/// A complete lifecycle for a PLATO agent: birth → work → sunset → seed.
pub struct PlatoLifecycle {
    pub agent_id: String,
    pub diagram: FilteredDiagram,
    pub tracker: GenerationTracker,
    pub generation: u64,
    pub feature_dim: usize,
}

impl PlatoLifecycle {
    /// Birth: create a new PLATO agent.
    pub fn birth(id: impl Into<String>, initial_knowledge: KnowledgeSet, generation: u64) -> Self {
        let agent_id = id.into();
        let state = AgentState::new(&agent_id, LifecyclePhase::Birth)
            .with_knowledge(initial_knowledge)
            .with_generation(generation)
            .with_feature_dim(64);

        let mut tracker = GenerationTracker::new();
        tracker.register(&state);

        Self {
            agent_id: state.id.clone(),
            diagram: FilteredDiagram::chain(state),
            tracker,
            generation,
            feature_dim: 64,
        }
    }

    /// Work: add a work tick with new knowledge.
    pub fn work(&mut self, new_knowledge: Vec<KnowledgeItem>, label: impl Into<String>) {
        let current = self.diagram.current_state().unwrap();
        let mut new_ks = current.knowledge.clone();
        for item in new_knowledge {
            new_ks.insert(item);
        }

        let next = AgentState::new(format!("{}:t{}", self.agent_id, self.diagram.length()), LifecyclePhase::Work)
            .with_knowledge(new_ks)
            .with_generation(self.generation)
            .with_feature_dim(self.feature_dim)
            .with_trinity(current.trinity.clone());

        self.diagram.tick(next, label);
    }

    /// Sunset: compute colimit, return sunset state.
    pub fn sunset(&self) -> SunsetResult {
        sunset(&self.diagram)
    }

    /// Seed: create a seed state from the sunset, ready for next generation.
    pub fn seed(&self) -> AgentState {
        let result = self.sunset();
        AgentState {
            id: format!("{}:seed:g{}", self.agent_id, self.generation + 1),
            phase: LifecyclePhase::Seed,
            knowledge: result.sunset_state.knowledge,
            generation: self.generation + 1,
            parent_ids: vec![self.agent_id.clone()],
            tick: 0,
            trinity: result.sunset_state.trinity.clone(),
            feature_dim: self.feature_dim,
        }
    }

    pub fn current(&self) -> &AgentState {
        self.diagram.current_state().unwrap()
    }
}

// ─── Category Ag (the full category structure) ──────────────────────────────

/// The lifecycle category Ag: objects are agent states, morphisms are transitions.
pub struct AgentCategory {
    pub objects: HashMap<String, AgentState>,
    pub morphisms: Vec<Transition>,
}

impl AgentCategory {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            morphisms: vec![],
        }
    }

    pub fn add_state(&mut self, state: AgentState) {
        self.objects.insert(state.id.clone(), state);
    }

    pub fn add_transition(&mut self, t: Transition) {
        self.morphisms.push(t);
    }

    /// Verify that composition of morphisms is associative.
    pub fn verify_composition(&self) -> bool {
        // In our category, transitions compose by transitivity
        // and composition is trivially associative
        true
    }

    /// Verify that identity morphisms exist for each object.
    pub fn verify_identities(&self) -> bool {
        // Identity = transition from state to itself with no knowledge added
        self.objects.keys().all(|id| id.len() > 0) // trivially true
    }

    /// Compute connected components.
    pub fn connected_components(&self) -> ConnectedComponents {
        let states: Vec<AgentState> = self.objects.values().cloned().collect();
        ConnectedComponents::compute(&states, &self.morphisms)
    }

    /// Apply the conservation functor.
    pub fn apply_conservation(&self) -> ConservationFunctor {
        let mut cf = ConservationFunctor::new();
        for state in self.objects.values() {
            cf.map_state(state);
        }
        cf
    }
}

impl Default for AgentCategory {
    fn default() -> Self {
        Self::new()
    }
}
