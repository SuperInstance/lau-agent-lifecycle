# lau-agent-lifecycle

> Categorical agent lifecycle — sunset as colimit, spawning as pullback, conservation via adjunction, kintsugi repair

Part of the **PLATO/LAU** mathematical agent framework.

---

## What This Does

This crate models the complete lifecycle of an agent — **birth → work → sunset → seed → spawn** — using category theory. Every operation has a precise mathematical meaning:

- **Sunset** (agent death) = **colimit** of the filtered lifetime diagram (lossless knowledge accumulation, modulo redundancy)
- **Spawn** (agent reproduction) = **pullback** in the category Ag (genetic crossover over a shared interface)
- **Conservation** = a **functor** C: Ag → Disc(X) that preserves invariants across all transitions
- **Repair** = the **kintsugi adjunction** (repair ⊣ break): golden-join finds the nearest valid state, and the repair is visible like golden cracks

Additional structures:

- **Trinity monad** (ethos/pathos/logos) — a triple structure T = (T, η, μ) governing the agent's moral, emotional, and rational dimensions
- **Connected components** π₀(Ag) with the **adjunction** π₀ ⊣ disc (Theorem 4.3)
- **Generation tracking** — genetic lineage, mutation counts, diversity metrics across agent generations
- **PLATO lifecycle application** — a complete lifecycle manager combining all structures

---

## Key Idea

> Agent death is not destruction — it's accumulation. The colimit of a lifetime diagram preserves *all* unique knowledge while removing redundancy. Agent birth is not creation ex nihilo — it's a pullback, a crossover of two parent genomes over a shared interface.

The category **Ag** has agent states as objects and knowledge-monotone transitions as morphisms. Knowledge only grows; it never shrinks. This monotonicity is the conservation law.

---

## Install

```toml
[dependencies]
lau-agent-lifecycle = { git = "https://github.com/SuperInstance/lau-agent-lifecycle" }
```

Or:

```bash
cargo add lau-agent-lifecycle
```

### Requirements

- Rust 2021 edition
- `serde` with `derive` feature
- `nalgebra` 0.33 (for feature vectors)

---

## Quick Start

```rust
use lau_agent_lifecycle::*;
use std::collections::BTreeSet;

// Birth: create a PLATO agent
let ks = KnowledgeSet::from_items(vec![
    KnowledgeItem::new("k0", "genesis knowledge", "core"),
]);
let mut plato = PlatoLifecycle::birth("agent-alpha", ks, 0);

// Work: accumulate knowledge over time
plato.work(vec![
    KnowledgeItem::new("k1", "learned calculus", "math"),
], "work-tick-1");
plato.work(vec![
    KnowledgeItem::new("k2", "learned topology", "math"),
    KnowledgeItem::new("k3", "learned logic", "logic"),
], "work-tick-2");

// Sunset: compute colimit (lossless accumulation)
let sunset = plato.sunset();
println!("Knowledge preserved: {}", sunset.unique_knowledge_preserved);
println!("Redundancy removed: {}", sunset.redundancy_removed);

// Seed: create a seed for next generation
let seed = plato.seed();
assert_eq!(seed.phase, LifecyclePhase::Seed);
assert_eq!(seed.generation, 1);

// Spawn: crossover two agents
let other_seed = /* ... another agent's seed ... */;
let interface = SharedInterface::new("math", BTreeSet::from(["k1".to_string()]));
let child = spawn(&seed, &other_seed, &interface, "gen1-child");
println!("Child generation: {}", child.child.generation);
println!("Crossover count: {}", child.crossover_count);
```

---

## API Reference

### Knowledge Types

| Type | Description |
|------|-------------|
| `KnowledgeItem` | ID, content, domain, generation, provenance chain |
| `KnowledgeSet` | BTreeMap-backed set with merge, dedup, domain filter, feature vector |

#### `KnowledgeSet` Methods

- `new()`, `from_items(vec)`, `insert(item) → bool`
- `merge(&other) → usize` — returns items added
- `contains(id) → bool`, `len()`, `is_empty()`
- `domain_filter(domain) → KnowledgeSet`
- `unique_against(&other) → KnowledgeSet`
- `deduplicate() → usize` — remove same-domain same-content duplicates
- `to_feature_vector(dim) → DVector<f64>` — hash-based feature vector

### Agent State

| Type | Description |
|------|-------------|
| `LifecyclePhase` | `Birth`, `Work`, `Sunset`, `Seed`, `Broken` |
| `AgentState` | id, phase, knowledge, generation, parents, tick, trinity, feature dimension |
| `ConservationSignature` | knowledge_count, generation, provenance count, phase, domain count |

#### `AgentState` Builder Methods

- `new(id, phase)` / `.with_knowledge(ks)` / `.with_generation(gen)` / `.with_parents(vec)` / `.with_tick(t)` / `.with_trinity(t)` / `.with_feature_dim(dim)`
- `feature_vector() → DVector<f64>` — knowledge set projected to feature space
- `conservation_signature() → ConservationSignature`

### Transitions and Diagrams

| Type | Description |
|------|-------------|
| `Transition` | source_id, target_id, knowledge_added, knowledge_preserved, label |
| `FilteredDiagram` | Chain of states + transitions with monotonicity verification |

#### `FilteredDiagram`

- `new()`, `chain(initial_state)`
- `tick(next_state, label) → &AgentState` — extend the chain
- `current_state() → Option<&AgentState>`
- `length() → usize`
- `verify_monotonicity() → bool` — check all transitions are knowledge-monotone
- `colimit() → AgentState` — accumulate all knowledge, deduplicate

### Sunset (Colimit)

```rust
pub fn sunset(diagram: &FilteredDiagram) -> SunsetResult
pub fn verify_knowledge_preservation(diagram: &FilteredDiagram, sunset: &AgentState) -> bool
```

`SunsetResult` contains: sunset_state, source_count, knowledge counts before/after, uniqueness flag, redundancy_removed.

### Spawn (Pullback)

```rust
pub fn spawn(parent1: &AgentState, parent2: &AgentState, interface: &SharedInterface, child_id: &str) -> SpawnResult
```

`SpawnResult`: child state, shared interface, unique counts per parent, crossover count.

### Conservation Functor

```rust
pub struct ConservationFunctor {
    mapping: HashMap<String, ConservationSignature>,
    discrete_cat: DiscreteCategory,
}
```

- `map_state(state) → ConservationSignature`
- `map_transition(t, source, target) → bool` — knowledge count non-decreasing
- `is_invariant(before, after) → bool` — domains don't shrink
- `verify_conservation(diagram) → bool`

### Connected Components

```rust
ConnectedComponents::compute(states, transitions) → ConnectedComponents
```

- `count() → usize`, `component_of(id) → Option<&BTreeSet<String>>`, `are_connected(a, b) → bool`

### Trinity Monad

```rust
pub struct TrinityState { ethos: f64, pathos: f64, logos: f64 }
```

- `new(ethos, pathos, logos)`, `balanced(value)`, `unit(value)`, `default()` (= 0.5, 0.5, 0.5)
- `multiply(outer, inner) → TrinityState` — monad multiplication μ
- `crossover(t1, t2) → TrinityState` — weighted average for spawning
- `mutate(δe, δp, δl) → TrinityState` — clamped to [0, 1]
- `magnitude() → f64` — L2 norm
- `verify_left_identity(v) → bool` / `verify_right_identity(t)` / `verify_associativity(t)` — monad law checks

### Kintsugi Adjunction

```rust
KintsugiAdjunction::break_state(state, damage_fraction, seed) → BrokenState
KintsugiAdjunction::repair(broken, original, golden_patch) → AgentState
KintsugiAdjunction::verify_adjunction(broken, original, golden_patch) → bool
```

### Generation Tracking

```rust
pub struct GenerationTracker { lineage: BTreeMap<String, GenealogyEntry>, max_generation: u64 }
```

- `register(state)`, `record_mutation(agent_id)`
- `diversity(states) → BTreeMap<u64, usize>` — distinct domains per generation
- `lineage_of(agent_id) → Vec<String>` — all ancestors
- `is_descendant_of(agent_id, ancestor_id) → bool`

### PLATO Lifecycle

```rust
let mut plato = PlatoLifecycle::birth("agent-1", initial_knowledge, 0);
plato.work(new_items, "label");
let sunset_result = plato.sunset();
let seed = plato.seed();
let current = plato.current();
```

### Agent Category

```rust
let mut cat = AgentCategory::new();
cat.add_state(state);
cat.add_transition(transition);
cat.verify_composition() → bool;
cat.verify_identities() → bool;
cat.connected_components() → ConnectedComponents;
cat.apply_conservation() → ConservationFunctor;
```

---

## How It Works

### The Category Ag

```
Objects:    Agent states (id, phase, knowledge, generation, ...)
Morphisms:  Knowledge-monotone transitions f: A → B where B.knowledge ⊇ A.knowledge
Composition: Transitivity (if f: A→B, g: B→C, then g∘f: A→C)
Identities:  id_A: A→A (trivial transition, no knowledge added)
```

### Filtered Lifetime Diagram

An agent's lifetime is a **filtered diagram**: a chain s₀ → s₁ → s₂ → ... where each step is knowledge-monotone. The colimit of this chain is the sunset state.

### Sunset (Colimit)

```
colim(s₀ → s₁ → ... → sₙ) = accumulate all knowledge, deduplicate
```

Every unique knowledge item from every state in the chain is preserved. Redundant items (same domain + same content) are merged. This is the "lossless accumulation modulo known redundancy" from Opus 4.8.

### Spawn (Pullback)

```
P₁ ×_A P₂ = { (p₁, p₂) | f₁(p₁) = f₂(p₂) in A }
```

In practice: given two parent agents and a shared interface (knowledge IDs they agree on), the child inherits:
1. All shared interface items (crossover, with dual provenance)
2. All unique items from each parent (with single provenance)

### Conservation Functor

C: Ag → Disc(X) maps each agent state to its conservation signature. The discrete category Disc(X) has only identity morphisms, so conservation signatures cannot change arbitrarily — they can only grow (knowledge count non-decreasing, domains non-shrinking).

---

## The Math

### Category Theory Foundations

**Definition (Category Ag):** Objects are agent states. A morphism f: A → B exists iff `B.knowledge ⊇ A.knowledge` (knowledge monotonicity).

**Theorem (Sunset as Colimit):** For a filtered diagram D = (s₀ → s₁ → ... → sₙ), the colimit colim(D) is the agent state with knowledge = ⋃ᵢ sᵢ.knowledge (after deduplication), satisfying the universal property that knowledge from every state is preserved.

**Theorem (Spawn as Pullback):** Given parent states P₁, P₂ and shared interface A, the pullback P₁ ×_A P₂ is the child state inheriting crossover items plus unique items from each parent.

**Theorem 4.3 (Adjunction π₀ ⊣ disc):** For any set X and category Ag:
```
Hom_Ag(disc(X), A) ≅ Hom_Set(X, π₀(A))
```
This means morphisms from disc-embedded states into A correspond to functions from X into the connected components of A.

### Trinity Monad

The trinity T = (T, η, μ) is a monad on the category of agent states:

- **Unit** η: A → T(A) maps a scalar to a balanced trinity
- **Multiplication** μ: T²(A) → T(A) flattens nested trinities via component-wise multiplication
- **Monad laws** (verified to within ε = 10⁻¹⁰):
  - Left identity: μ ∘ Tη ≈ id
  - Right identity: μ ∘ ηT ≈ id
  - Associativity: μ ∘ Tμ ≈ μ ∘ μT

### Kintsugi Adjunction

The adjunction repair ⊣ break satisfies:
```
Hom_Ag(repair(B), A) ≅ Hom_Ag_broken(B, break(A))
```

`break` introduces controlled damage (removes a fraction of knowledge items). `repair` applies a golden patch to recover. The golden join is visible — the repair is not erased, it's celebrated.

### Feature Vectors

Knowledge sets are projected to ℝᵈ via hash-based feature vectors:

```
v[i] = |{ k ∈ K : hash(k.id) mod d = i }|
```

This is a bag-of-words style embedding in the knowledge domain.

---

## Test Suite

**73 integration tests** covering:

- KnowledgeSet: insert, dedup, merge, domain filter, unique-against, redundancy, feature vectors, empty set
- AgentState: creation, builder methods, conservation signature, phase display
- Transitions: monotone and non-monotone
- FilteredDiagram: chain construction, monotonicity verification, colimit computation
- Sunset: knowledge preservation, redundancy removal, verification
- Spawn: basic crossover, inherits all, generation increment
- ConservationFunctor: state mapping, domain preservation, diagram verification
- ConnectedComponents: single, linked, separate; disc_embed; adjunction verification
- Trinity: creation, unit, multiply, crossover, mutation (with clamping), magnitude, monad laws (left/right identity, associativity), equality
- Kintsugi: break, repair, adjunction verification, zero damage
- GenerationTracker: register, mutation counting, diversity, lineage, descendant checks, multi-generation
- AgentCategory: creation, connected components, conservation, laws
- PLATO lifecycle: birth, work ticks, sunset, seed, full lifecycle (birth → work → sunset → seed → spawn)
- Serialization: KnowledgeSet, AgentState, ConnectedComponents, GenerationTracker, BrokenState
- Edge cases: empty knowledge, shared interface, long chain colimit (11 states)

Run: `cargo test`

---

## License

MIT
