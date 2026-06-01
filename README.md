# lau-agent-lifecycle

A categorical agent lifecycle framework — sunset as colimit, spawning as pullback, conservation via adjunction, kintsugi repair — implemented in Rust.

Models the complete birth → work → sunset → seed → spawn lifecycle of agents using category theory, with knowledge preservation, genetic lineage tracking, and formal verification.

---

## What This Does

This crate provides a rigorous categorical framework for AI agent lifecycles:

- **Agent states** — Objects in a category **Ag** with knowledge sets, generation tracking, and trinity (ethos/pathos/logos) state
- **Knowledge-monotone transitions** — Morphisms that only add knowledge, never lose it
- **Filtered lifetime diagrams** — Chains of agent states connected by monotone transitions
- **Sunset (colimit)** — Lossless accumulation of all knowledge modulo redundancy
- **Spawn (pullback)** — Genetic crossover: two parents produce a child agreeing on a shared interface
- **Conservation functor** — C: Ag → Disc(X) preserving invariants across all transitions
- **Connected components** — π₀(Ag) grouping agents by reachability, with adjunction π₀ ⊣ disc
- **Trinity monad** — T = (T, η, μ) capturing ethos/pathos/logos with monad law verification
- **Kintsugi adjunction** — repair ⊣ break: controlled damage with golden repair to nearest valid state
- **Generation tracking** — Genetic lineage, mutation counting, diversity metrics
- **PLATO lifecycle** — Complete birth → work → sunset → seed → spawn application

## Key Idea

Agent lifecycles form a **category** where:
- **Objects** are agent states (with knowledge, generation, trinity)
- **Morphisms** are knowledge-monotone transitions (knowledge only grows)
- **Sunset** is the **colimit** of a filtered diagram — accumulating everything losslessly
- **Spawning** is a **pullback** — two parents agreeing on a shared interface produce a child
- **Conservation** is a **functor** to a discrete category — invariants are preserved

This categorical structure gives us formal guarantees: knowledge preservation is a theorem, not a hope.

## Install

Add to your `Cargo.toml`:

```toml
[dependencies]
lau-agent-lifecycle = "0.1.0"
```

Requires Rust 2021 edition or later.

## Quick Start

```rust
use lau_agent_lifecycle::*;
use std::collections::BTreeSet;

// Birth: create a PLATO agent with initial knowledge
let ks = KnowledgeSet::from_items(vec![
    KnowledgeItem::new("k0", "genesis", "core"),
]);
let mut agent = PlatoLifecycle::birth("plato-1", ks, 0);

// Work: add knowledge through work ticks
agent.work(vec![
    KnowledgeItem::new("k1", "learned math", "math"),
], "tick-1");
agent.work(vec![
    KnowledgeItem::new("k2", "learned logic", "logic"),
], "tick-2");

// Sunset: colimit — accumulate all knowledge
let sunset = agent.sunset();
assert!(sunset.unique_knowledge_preserved);

// Seed: prepare for next generation
let seed = agent.seed();
assert_eq!(seed.generation, 1);
assert_eq!(seed.knowledge.len(), 3);

// Spawn: pullback — crossover two agents
let p1 = AgentState::new("p1", LifecyclePhase::Sunset)
    .with_knowledge(KnowledgeSet::from_items(vec![
        KnowledgeItem::new("k1", "a", "math"),
    ]));
let p2 = AgentState::new("p2", LifecyclePhase::Sunset)
    .with_knowledge(KnowledgeSet::from_items(vec![
        KnowledgeItem::new("k2", "b", "logic"),
    ]));
let result = spawn(&p1, &p2, &SharedInterface::empty(), "child");
assert_eq!(result.child.knowledge.len(), 2);
assert_eq!(result.child.generation, 1);

// Kintsugi: break and repair with golden patches
let broken = KintsugiAdjunction::break_state(&p1, 0.5, 42);
let golden = KnowledgeSet::from_items(vec![
    KnowledgeItem::new("golden-1", "repair patch", "recovery"),
]);
let repaired = KintsugiAdjunction::repair(&broken, &p1, &golden);
assert_eq!(repaired.phase, LifecyclePhase::Work);
```

## API Reference

### Knowledge

| Type | Description |
|------|-------------|
| `KnowledgeItem` | A knowledge item with id, content, domain, generation, provenance |
| `KnowledgeSet` | Deduplicating set of knowledge items, supports merge and filtering |

Key methods on `KnowledgeSet`:
- `insert(item)` — Add an item (returns false if duplicate)
- `merge(&other)` — Merge another set, returns count of new items
- `domain_filter(domain)` — Filter to items in a given domain
- `unique_against(&other)` — Items not in the other set
- `deduplicate()` — Remove items with same content/domain
- `to_feature_vector(dim)` — Convert to a numerical vector

### Agent States

| Type | Description |
|------|-------------|
| `LifecyclePhase` | Birth, Work, Sunset, Seed, Broken |
| `AgentState` | An object in category Ag |
| `ConservationSignature` | Invariant signature preserved by transitions |
| `Transition` | A knowledge-monotone morphism s₁ → s₂ |

`AgentState` supports builder pattern:
```rust
let state = AgentState::new("agent-1", LifecyclePhase::Birth)
    .with_knowledge(ks)
    .with_generation(2)
    .with_parents(vec!["p1".to_string(), "p2".to_string()])
    .with_trinity(TrinityState::new(0.8, 0.6, 0.9))
    .with_feature_dim(128);
```

### Filtered Diagrams

| Type | Description |
|------|-------------|
| `FilteredDiagram` | Chain of states connected by monotone transitions |

Methods:
- `chain(initial)` — Start a diagram from an initial state
- `tick(next, label)` — Extend with a new state
- `verify_monotonicity()` — Check all transitions are knowledge-monotone
- `colimit()` — Compute the sunset state

### Sunset (Colimit)

| Function | Description |
|----------|-------------|
| `sunset(&diagram)` | Compute colimit with full verification |
| `verify_knowledge_preservation(diagram, sunset)` | Verify no knowledge lost |

Returns `SunsetResult` with:
- `sunset_state` — The accumulated agent state
- `unique_knowledge_preserved` — Whether all unique items survived
- `redundancy_removed` — Count of deduplicated items

### Spawn (Pullback)

| Function | Description |
|----------|-------------|
| `spawn(p1, p2, interface, child_id)` | Pullback crossover |
| `SharedInterface::new(domain, ids)` | Define the shared agreement |

Returns `SpawnResult` with:
- `child` — The new agent state
- `crossover_count` — Items from the shared interface
- `inherited_from_p1/p2` — Counts from each parent

### Conservation Functor

| Type | Description |
|------|-------------|
| `ConservationFunctor` | C: Ag → Disc(X) |
| `DiscreteCategory` | Objects with only identity morphisms |

Methods:
- `map_state(&state)` — Map to conservation signature
- `map_transition(t, src, tgt)` — Verify conservation law
- `verify_conservation(&diagram)` — Check entire diagram

### Connected Components

| Type | Description |
|------|-------------|
| `ConnectedComponents` | π₀(Ag) grouping by reachability |
| `disc_embed(ids, phase)` | The disc functor: Set → Ag |

Methods:
- `compute(states, transitions)` — Union-Find based computation
- `count()` — Number of components
- `are_connected(a, b)` — Check reachability

### Trinity Monad

| Type | Description |
|------|-------------|
| `TrinityState` | Ethos × Pathos × Logos triple |

The trinity monad `T = (T, η, μ)`:
- `unit(value)` — η: A → T(A), embed into balanced trinity
- `multiply(outer, inner)` — μ: T²(A) → T(A), flatten nested trinity
- `crossover(t1, t2)` — Average during spawning
- `mutate(Δe, Δp, Δl)` — Add noise (clamped to [0,1])
- `verify_left_identity/right_identity/associativity` — Check monad laws

### Kintsugi Adjunction

| Type | Description |
|------|-------------|
| `BrokenState` | An agent state with controlled damage |
| `KintsugiAdjunction` | The repair ⊣ break adjunction |

Methods:
- `break_state(&state, fraction, seed)` — Introduce controlled damage
- `repair(&broken, &original, &golden_patch)` — Golden join to nearest valid state
- `verify_adjunction(...)` — Check the adjunction property

### Generation Tracking

| Type | Description |
|------|-------------|
| `GenerationTracker` | Tracks genetic lineage and diversity |
| `GenealogyEntry` | Per-agent record with generation, parents, mutations |

Methods:
- `register(&state)` — Register an agent
- `record_mutation(id)` — Track a mutation
- `diversity(&states)` — Distinct domains per generation
- `lineage_of(id)` — All ancestors
- `is_descendant_of(id, ancestor)` — Check ancestry

### PLATO Lifecycle

| Type | Description |
|------|-------------|
| `PlatoLifecycle` | Complete birth → work → sunset → seed cycle |
| `AgentCategory` | The full category Ag |

```rust
let mut plato = PlatoLifecycle::birth("agent", initial_knowledge, 0);
plato.work(vec![item1], "tick-1");
plato.work(vec![item2], "tick-2");
let sunset = plato.sunset();
let seed = plato.seed();
```

## How It Works

The framework builds on categorical abstractions:

1. **Knowledge management**: `KnowledgeSet` is a deduplicating set keyed by ID, with domain filtering, merge operations, and feature vector conversion. Items carry provenance chains for tracking where knowledge originated.

2. **Category Ag**: Objects are `AgentState`s (knowledge + generation + trinity + metadata). Morphisms are `Transition`s that are *knowledge-monotone* — the target's knowledge is a superset of the source's. This is enforced by `is_monotone()`.

3. **Filtered diagrams**: A `FilteredDiagram` is a chain s₀ → s₁ → s₂ → ... where each step adds knowledge. The diagram tracks the full history and supports monotonicity verification.

4. **Sunset as colimit**: The colimit of a filtered diagram accumulates all knowledge from every state, then deduplicates. The result is a single state containing every unique knowledge item. Verification checks that nothing was lost.

5. **Spawn as pullback**: Given two parent agents P₁ and P₂ and a shared interface A (knowledge they agree on), the pullback P₁ ×_A P₂ produces a child that inherits the shared interface with dual provenance, plus each parent's unique knowledge.

6. **Conservation functor**: Maps each agent state to a `ConservationSignature` (knowledge count, generation, domain count, phase). This maps into a discrete category (only identity morphisms), so invariants are frozen. The functor verifies that domains never shrink across transitions.

7. **Connected components π₀**: Uses union-find to group agents by reachability through transitions. The adjunction π₀ ⊣ disc says that morphisms from disc-embedded states into Ag correspond to functions into the connected components.

8. **Trinity monad**: Each agent carries an (ethos, pathos, logos) triple. The monad structure gives unit (embed), multiplication (flatten), and satisfies the monad laws (left/right identity, associativity). Crossover during spawn averages the parents' trinities.

9. **Kintsugi adjunction**: `break` introduces controlled damage (removes a fraction of knowledge), `repair` applies a golden patch to find the nearest valid state. The adjunction `repair ⊣ break` means: maps from the repaired state to any valid state correspond to maps from the broken state to the broken version of that valid state.

## The Math

The categorical structure:

| Category Theory | Agent Lifecycle |
|-----------------|-----------------|
| Object | Agent state (knowledge + metadata) |
| Morphism | Knowledge-monotone transition |
| Identity | No-op transition |
| Composition | Sequential transitions |
| Colimit (filtered) | Sunset (knowledge accumulation) |
| Pullback | Spawn (genetic crossover) |
| Functor C: Ag → Disc(X) | Conservation of invariants |
| π₀(Ag) | Connected components (reachability) |
| π₀ ⊣ disc | Adjunction (Theorem 4.3) |
| Monad (T, η, μ) | Trinity (ethos/pathos/logos) |
| Adjunction repair ⊣ break | Kintsugi (golden repair) |

**Knowledge monotonicity** is the key invariant: in category Ag, morphisms only add knowledge. This makes Ag a *poset-enriched* category where the ordering is knowledge inclusion.

The **colimit** of a filtered diagram exists because knowledge sets are closed under directed unions — the colimit is simply the union of all knowledge, modulo deduplication.

The **pullback** P₁ ×_A P₂ exists when both parents can map to the shared interface A. The child inherits the interface items (with dual provenance) plus each parent's unique contributions.

**Conservation** means: certain quantities (knowledge count, domain diversity) are non-decreasing under all transitions. The conservation functor freezes these into a discrete category where only identity morphisms exist.

## Tests

The crate contains **73 tests** (10 unit + 63 integration) covering:
- Knowledge set operations (insert, merge, filter, deduplicate, feature vectors)
- Agent state creation and conservation signatures
- Transition monotonicity (positive and negative cases)
- Filtered diagram chains and monotonicity verification
- Sunset colimit with knowledge preservation and redundancy removal
- Spawn pullback with crossover, generation increment, and full inheritance
- Conservation functor mapping and diagram verification
- Connected components (single, linked, separate)
- Disc embedding and adjunction verification
- Trinity monad operations and law verification (left/right identity, associativity)
- Kintsugi break/repair and adjunction verification
- Generation tracking (registration, mutation, diversity, lineage)
- PLATO full lifecycle (birth → work → sunset → seed → spawn)
- Serialization round-trips for all major types

Run with:

```bash
cargo test
```

## License

MIT
