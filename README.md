# lau-agent-lifecycle

> Categorical agent lifecycle — sunset as colimit, spawning as pullback, conservation via adjunction, kintsugi repair

## What This Does

Categorical agent lifecycle — sunset as colimit, spawning as pullback, conservation via adjunction, kintsugi repair. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-agent-lifecycle
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_agent_lifecycle::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub struct KnowledgeItem 
    pub fn new(id: impl Into<String>, content: impl Into<String>, domain: impl Into<String>) -> Self 
    pub fn with_generation(mut self, gen: u64) -> Self 
    pub fn with_provenance(mut self, p: Vec<String>) -> Self 
    pub fn is_redundant_with(&self, other: &KnowledgeItem) -> bool 
pub struct KnowledgeSet 
    pub fn new() -> Self 
    pub fn from_items(items: Vec<KnowledgeItem>) -> Self 
    pub fn insert(&mut self, item: KnowledgeItem) -> bool 
    pub fn merge(&mut self, other: &KnowledgeSet) -> usize 
    pub fn contains(&self, id: &str) -> bool 
    pub fn len(&self) -> usize 
    pub fn is_empty(&self) -> bool 
    pub fn items(&self) -> impl Iterator<Item = &KnowledgeItem> 
    pub fn domain_filter(&self, domain: &str) -> KnowledgeSet 
    pub fn unique_against(&self, other: &KnowledgeSet) -> KnowledgeSet 
    pub fn to_feature_vector(&self, dimension: usize) -> DVector<f64> 
    pub fn deduplicate(&mut self) -> usize 
pub enum LifecyclePhase 
pub struct AgentState 
    pub fn new(id: impl Into<String>, phase: LifecyclePhase) -> Self 
    pub fn with_knowledge(mut self, ks: KnowledgeSet) -> Self 
    pub fn with_generation(mut self, gen: u64) -> Self 
    pub fn with_parents(mut self, parents: Vec<String>) -> Self 
    pub fn with_tick(mut self, tick: u64) -> Self 
    pub fn with_trinity(mut self, t: TrinityState) -> Self 
    pub fn with_feature_dim(mut self, dim: usize) -> Self 
    pub fn feature_vector(&self) -> DVector<f64> 
    pub fn conservation_signature(&self) -> ConservationSignature 
pub struct ConservationSignature 
pub struct Transition 
    pub fn new(source: &AgentState, target: &AgentState, label: impl Into<String>) -> Self 
    pub fn is_monotone(&self, source: &AgentState, target: &AgentState) -> bool 
pub struct FilteredDiagram 
    pub fn new() -> Self 
    pub fn chain(initial: AgentState) -> Self 
    pub fn tick(&mut self, mut next: AgentState, label: impl Into<String>) -> &AgentState 
    pub fn current_state(&self) -> Option<&AgentState> 
    pub fn length(&self) -> usize 
    pub fn verify_monotonicity(&self) -> bool 
    pub fn colimit(&self) -> AgentState 
pub struct SunsetResult 
pub fn sunset(diagram: &FilteredDiagram) -> SunsetResult 
pub fn verify_knowledge_preservation(diagram: &FilteredDiagram, sunset_state: &AgentState) -> bool 
pub struct SharedInterface 
    pub fn new(domain: impl Into<String>, ids: BTreeSet<String>) -> Self 
    pub fn empty() -> Self 
    pub fn len(&self) -> usize 
    pub fn is_empty(&self) -> bool 
pub struct SpawnResult 
pub fn spawn(
pub struct DiscreteCategory 
    pub fn new() -> Self 
    pub fn insert(&mut self, obj: String) 
    pub fn contains(&self, obj: &str) -> bool 
    pub fn len(&self) -> usize 
    pub fn is_empty(&self) -> bool 
pub struct ConservationFunctor 
    pub fn new() -> Self 
    pub fn map_state(&mut self, state: &AgentState) -> ConservationSignature 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**73 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
