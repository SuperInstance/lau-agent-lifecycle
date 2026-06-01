#[cfg(test)]
mod tests {
    use lau_agent_lifecycle::*;
    use std::collections::BTreeSet;

    // ─── KnowledgeSet Tests ──────────────────────────────────────────────

    #[test]
    fn test_knowledge_set_insert() {
        let mut ks = KnowledgeSet::new();
        let item = KnowledgeItem::new("k1", "test content", "domain_a");
        assert!(ks.insert(item));
        assert_eq!(ks.len(), 1);
    }

    #[test]
    fn test_knowledge_set_no_duplicates() {
        let mut ks = KnowledgeSet::new();
        ks.insert(KnowledgeItem::new("k1", "content", "d"));
        ks.insert(KnowledgeItem::new("k1", "content", "d"));
        assert_eq!(ks.len(), 1);
    }

    #[test]
    fn test_knowledge_set_merge() {
        let mut ks1 = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "d"),
        ]);
        let ks2 = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k2", "b", "d"),
            KnowledgeItem::new("k1", "a", "d"), // duplicate
        ]);
        let added = ks1.merge(&ks2);
        assert_eq!(added, 1);
        assert_eq!(ks1.len(), 2);
    }

    #[test]
    fn test_knowledge_set_domain_filter() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "math"),
            KnowledgeItem::new("k2", "b", "math"),
            KnowledgeItem::new("k3", "c", "logic"),
        ]);
        let math = ks.domain_filter("math");
        assert_eq!(math.len(), 2);
    }

    #[test]
    fn test_knowledge_set_unique_against() {
        let ks1 = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "d"),
            KnowledgeItem::new("k2", "b", "d"),
        ]);
        let ks2 = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k2", "b", "d"),
        ]);
        let unique = ks1.unique_against(&ks2);
        assert_eq!(unique.len(), 1);
        assert!(unique.contains("k1"));
    }

    #[test]
    fn test_knowledge_item_redundancy() {
        let k1 = KnowledgeItem::new("k1", "same content", "math");
        let k2 = KnowledgeItem::new("k2", "same content", "math");
        assert!(k1.is_redundant_with(&k2));

        let k3 = KnowledgeItem::new("k3", "different", "math");
        assert!(!k1.is_redundant_with(&k3));
    }

    #[test]
    fn test_knowledge_set_deduplicate() {
        let mut ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "same", "d"),
            KnowledgeItem::new("k2", "same", "d"),
        ]);
        let removed = ks.deduplicate();
        assert_eq!(removed, 1);
        assert_eq!(ks.len(), 1);
    }

    #[test]
    fn test_knowledge_feature_vector() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "d"),
            KnowledgeItem::new("k2", "b", "d"),
        ]);
        let v = ks.to_feature_vector(16);
        assert_eq!(v.len(), 16);
        // At least some non-zero entries
        assert!(v.iter().any(|x| *x != 0.0));
    }

    // ─── AgentState Tests ────────────────────────────────────────────────

    #[test]
    fn test_agent_state_creation() {
        let state = AgentState::new("agent-1", LifecyclePhase::Birth);
        assert_eq!(state.id, "agent-1");
        assert_eq!(state.phase, LifecyclePhase::Birth);
        assert_eq!(state.generation, 0);
        assert!(state.knowledge.is_empty());
    }

    #[test]
    fn test_agent_state_with_knowledge() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "test", "d"),
        ]);
        let state = AgentState::new("a", LifecyclePhase::Work).with_knowledge(ks);
        assert_eq!(state.knowledge.len(), 1);
    }

    #[test]
    fn test_agent_state_conservation_signature() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "math"),
            KnowledgeItem::new("k2", "b", "logic"),
        ]);
        let state = AgentState::new("a", LifecyclePhase::Work).with_knowledge(ks);
        let sig = state.conservation_signature();
        assert_eq!(sig.knowledge_count, 2);
        assert_eq!(sig.domains, 2);
    }

    #[test]
    fn test_lifecycle_phase_display() {
        assert_eq!(format!("{}", LifecyclePhase::Birth), "Birth");
        assert_eq!(format!("{}", LifecyclePhase::Sunset), "Sunset");
        assert_eq!(format!("{}", LifecyclePhase::Seed), "Seed");
    }

    // ─── Transition Tests ────────────────────────────────────────────────

    #[test]
    fn test_transition_monotone() {
        let s1 = AgentState::new("s1", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "d"),
            ]));
        let s2 = AgentState::new("s2", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "d"),
                KnowledgeItem::new("k2", "b", "d"),
            ]));
        let t = Transition::new(&s1, &s2, "tick");
        assert!(t.is_monotone(&s1, &s2));
        assert_eq!(t.knowledge_preserved, 1);
        assert_eq!(t.knowledge_added, 1);
    }

    #[test]
    fn test_transition_non_monotone() {
        let s1 = AgentState::new("s1", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "d"),
                KnowledgeItem::new("k2", "b", "d"),
            ]));
        let s2 = AgentState::new("s2", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "d"),
            ]));
        let t = Transition::new(&s1, &s2, "lossy");
        assert!(!t.is_monotone(&s1, &s2));
    }

    // ─── FilteredDiagram Tests ───────────────────────────────────────────

    #[test]
    fn test_filtered_diagram_chain() {
        let s0 = AgentState::new("a:t0", LifecyclePhase::Birth)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "init", "d"),
            ]));
        let mut diag = FilteredDiagram::chain(s0);
        assert_eq!(diag.length(), 1);

        let s1 = AgentState::new("a:t1", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "init", "d"),
                KnowledgeItem::new("k1", "new", "d"),
            ]));
        diag.tick(s1, "work-tick");
        assert_eq!(diag.length(), 2);
        assert_eq!(diag.current_state().unwrap().tick, 1);
    }

    #[test]
    fn test_filtered_diagram_monotonicity() {
        let s0 = AgentState::new("a:t0", LifecyclePhase::Birth)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "init", "d"),
            ]));
        let mut diag = FilteredDiagram::chain(s0);

        let s1 = AgentState::new("a:t1", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "init", "d"),
                KnowledgeItem::new("k1", "learned", "d"),
            ]));
        diag.tick(s1, "learn");
        assert!(diag.verify_monotonicity());
    }

    #[test]
    fn test_filtered_diagram_colimit() {
        let s0 = AgentState::new("a:t0", LifecyclePhase::Birth)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "init", "d"),
            ]));
        let mut diag = FilteredDiagram::chain(s0);

        let s1 = AgentState::new("a:t1", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "init", "d"),
                KnowledgeItem::new("k1", "learned", "d"),
            ]));
        diag.tick(s1, "work");

        let colimit = diag.colimit();
        assert_eq!(colimit.phase, LifecyclePhase::Sunset);
        assert_eq!(colimit.knowledge.len(), 2);
    }

    // ─── Sunset (Colimit) Tests ──────────────────────────────────────────

    #[test]
    fn test_sunset_preserves_knowledge() {
        let s0 = AgentState::new("a:t0", LifecyclePhase::Birth)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "a", "d"),
                KnowledgeItem::new("k1", "b", "d"),
            ]));
        let mut diag = FilteredDiagram::chain(s0);

        let s1 = AgentState::new("a:t1", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "a", "d"),
                KnowledgeItem::new("k1", "b", "d"),
                KnowledgeItem::new("k2", "c", "d"),
            ]));
        diag.tick(s1, "work");

        let result = sunset(&diag);
        assert!(result.unique_knowledge_preserved);
        assert_eq!(result.knowledge_after_total, 3);
    }

    #[test]
    fn test_sunset_removes_redundancy() {
        let s0 = AgentState::new("a:t0", LifecyclePhase::Birth)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "same", "d"),
            ]));
        let mut diag = FilteredDiagram::chain(s0);

        let s1 = AgentState::new("a:t1", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "same", "d"),
                KnowledgeItem::new("k2", "same", "d"), // redundant
            ]));
        diag.tick(s1, "work");

        let result = sunset(&diag);
        assert!(result.redundancy_removed > 0);
    }

    #[test]
    fn test_sunset_verification() {
        let s0 = AgentState::new("a:t0", LifecyclePhase::Birth)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "a", "d"),
            ]));
        let mut diag = FilteredDiagram::chain(s0);

        let s1 = AgentState::new("a:t1", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "a", "d"),
                KnowledgeItem::new("k1", "b", "d"),
            ]));
        diag.tick(s1, "work");

        let colimit = diag.colimit();
        assert!(verify_knowledge_preservation(&diag, &colimit));
    }

    // ─── Spawn (Pullback) Tests ──────────────────────────────────────────

    #[test]
    fn test_spawn_basic() {
        let p1 = AgentState::new("p1", LifecyclePhase::Sunset)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "math"),
                KnowledgeItem::new("k2", "b", "math"),
            ]));
        let p2 = AgentState::new("p2", LifecyclePhase::Sunset)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k2", "b", "math"),
                KnowledgeItem::new("k3", "c", "logic"),
            ]));
        let interface = SharedInterface::new("math", BTreeSet::from(["k2".to_string()]));

        let result = spawn(&p1, &p2, &interface, "child-1");
        assert_eq!(result.child.phase, LifecyclePhase::Birth);
        assert_eq!(result.child.generation, 1);
        assert_eq!(result.child.parent_ids, vec!["p1", "p2"]);
    }

    #[test]
    fn test_spawn_crossover() {
        let p1 = AgentState::new("p1", LifecyclePhase::Sunset)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("shared", "common", "d"),
                KnowledgeItem::new("p1-unique", "a", "d"),
            ]));
        let p2 = AgentState::new("p2", LifecyclePhase::Sunset)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("shared", "common", "d"),
                KnowledgeItem::new("p2-unique", "b", "d"),
            ]));
        let interface = SharedInterface::new("d", BTreeSet::from(["shared".to_string()]));

        let result = spawn(&p1, &p2, &interface, "child");
        assert_eq!(result.crossover_count, 1);
        assert!(result.child.knowledge.contains("shared"));
        assert!(result.child.knowledge.contains("p1-unique"));
        assert!(result.child.knowledge.contains("p2-unique"));
    }

    #[test]
    fn test_spawn_inherits_all() {
        let p1 = AgentState::new("p1", LifecyclePhase::Sunset)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "d"),
            ]));
        let p2 = AgentState::new("p2", LifecyclePhase::Sunset)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k2", "b", "d"),
            ]));
        let interface = SharedInterface::empty();

        let result = spawn(&p1, &p2, &interface, "child");
        assert_eq!(result.child.knowledge.len(), 2);
    }

    #[test]
    fn test_spawn_generation_increments() {
        let p1 = AgentState::new("p1", LifecyclePhase::Sunset).with_generation(3);
        let p2 = AgentState::new("p2", LifecyclePhase::Sunset).with_generation(5);
        let result = spawn(&p1, &p2, &SharedInterface::empty(), "child");
        assert_eq!(result.child.generation, 6);
    }

    // ─── Conservation Functor Tests ──────────────────────────────────────

    #[test]
    fn test_conservation_functor_maps_state() {
        let state = AgentState::new("a", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "math"),
            ]));
        let mut cf = ConservationFunctor::new();
        let sig = cf.map_state(&state);
        assert_eq!(sig.knowledge_count, 1);
        assert_eq!(sig.domains, 1);
        assert!(cf.discrete_cat.len() > 0);
    }

    #[test]
    fn test_conservation_preserves_domains() {
        let ks1 = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "math"),
        ]);
        let ks2 = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "math"),
            KnowledgeItem::new("k2", "b", "logic"),
        ]);
        let s1 = AgentState::new("s1", LifecyclePhase::Work).with_knowledge(ks1);
        let s2 = AgentState::new("s2", LifecyclePhase::Work).with_knowledge(ks2);

        let mut cf = ConservationFunctor::new();
        cf.map_state(&s1);
        cf.map_state(&s2);
        assert!(cf.is_invariant(&s1, &s2));
    }

    #[test]
    fn test_conservation_verifies_diagram() {
        let s0 = AgentState::new("s0", LifecyclePhase::Birth)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "init", "d"),
            ]));
        let mut diag = FilteredDiagram::chain(s0);
        let s1 = AgentState::new("s1", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "init", "d"),
                KnowledgeItem::new("k1", "more", "d"),
            ]));
        diag.tick(s1, "grow");

        let mut cf = ConservationFunctor::new();
        assert!(cf.verify_conservation(&diag));
    }

    // ─── Connected Components Tests ──────────────────────────────────────

    #[test]
    fn test_connected_components_single() {
        let states = vec![AgentState::new("a", LifecyclePhase::Work)];
        let pi0 = ConnectedComponents::compute(&states, &[]);
        assert_eq!(pi0.count(), 1);
    }

    #[test]
    fn test_connected_components_linked() {
        let states = vec![
            AgentState::new("a", LifecyclePhase::Work),
            AgentState::new("b", LifecyclePhase::Work),
        ];
        let transitions = vec![Transition {
            source_id: "a".into(),
            target_id: "b".into(),
            knowledge_added: 0,
            knowledge_preserved: 0,
            label: "t".into(),
        }];
        let pi0 = ConnectedComponents::compute(&states, &transitions);
        assert_eq!(pi0.count(), 1);
        assert!(pi0.are_connected("a", "b"));
    }

    #[test]
    fn test_connected_components_separate() {
        let states = vec![
            AgentState::new("a", LifecyclePhase::Work),
            AgentState::new("b", LifecyclePhase::Work),
        ];
        let pi0 = ConnectedComponents::compute(&states, &[]);
        assert_eq!(pi0.count(), 2);
        assert!(!pi0.are_connected("a", "b"));
    }

    #[test]
    fn test_disc_embed() {
        let ids = vec!["x".to_string(), "y".to_string()];
        let states = disc_embed(&ids, LifecyclePhase::Birth);
        assert_eq!(states.len(), 2);
        assert_eq!(states[0].id, "x");
    }

    #[test]
    fn test_adjunction_verification() {
        let embedded = disc_embed(&["e1".to_string()], LifecyclePhase::Birth);
        let states = vec![AgentState::new("a", LifecyclePhase::Work)];
        let transitions = vec![Transition {
            source_id: "a".into(),
            target_id: "a".into(),
            knowledge_added: 0,
            knowledge_preserved: 0,
            label: "id".into(),
        }];
        assert!(verify_adjunction(&embedded, &states, &transitions));
    }

    // ─── Trinity Monad Tests ─────────────────────────────────────────────

    #[test]
    fn test_trinity_creation() {
        let t = TrinityState::new(0.8, 0.6, 0.9);
        assert!((t.ethos - 0.8).abs() < 1e-10);
        assert!((t.pathos - 0.6).abs() < 1e-10);
        assert!((t.logos - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_trinity_unit() {
        let t = TrinityState::unit(0.5);
        assert!((t.ethos - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_trinity_multiply() {
        let t1 = TrinityState::new(0.5, 0.5, 0.5);
        let t2 = TrinityState::new(0.8, 0.8, 0.8);
        let result = TrinityState::multiply(&t1, &t2);
        assert!((result.ethos - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_trinity_crossover() {
        let t1 = TrinityState::new(1.0, 0.0, 0.5);
        let t2 = TrinityState::new(0.0, 1.0, 0.5);
        let child = TrinityState::crossover(&t1, &t2);
        assert!((child.ethos - 0.5).abs() < 1e-10);
        assert!((child.pathos - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_trinity_mutation() {
        let t = TrinityState::new(0.5, 0.5, 0.5);
        let mutated = t.mutate(0.3, -0.2, 0.1);
        assert!((mutated.ethos - 0.8).abs() < 1e-10);
        assert!((mutated.pathos - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_trinity_mutation_clamped() {
        let t = TrinityState::new(0.9, 0.1, 0.5);
        let mutated = t.mutate(0.5, -0.5, 0.0);
        assert!((mutated.ethos - 1.0).abs() < 1e-10); // clamped
        assert!((mutated.pathos - 0.0).abs() < 1e-10); // clamped
    }

    #[test]
    fn test_trinity_magnitude() {
        let t = TrinityState::new(1.0, 0.0, 0.0);
        assert!((t.magnitude() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_trinity_left_identity() {
        assert!(TrinityState::verify_left_identity(0.7));
    }

    #[test]
    fn test_trinity_right_identity() {
        let t = TrinityState::new(0.6, 0.4, 0.8);
        assert!(TrinityState::verify_right_identity(&t));
    }

    #[test]
    fn test_trinity_associativity() {
        let t = TrinityState::new(0.5, 0.6, 0.7);
        assert!(TrinityState::verify_associativity(&t));
    }

    #[test]
    fn test_trinity_equality() {
        let t1 = TrinityState::new(0.5, 0.5, 0.5);
        let t2 = TrinityState::new(0.5, 0.5, 0.5);
        assert_eq!(t1, t2);
    }

    // ─── Kintsugi Adjunction Tests ──────────────────────────────────────

    #[test]
    fn test_kintsugi_break() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "d"),
            KnowledgeItem::new("k2", "b", "d"),
            KnowledgeItem::new("k3", "c", "d"),
        ]);
        let state = AgentState::new("a", LifecyclePhase::Work)
            .with_knowledge(ks)
            .with_feature_dim(16);
        let broken = KintsugiAdjunction::break_state(&state, 0.5, 42);
        assert!(broken.lost_knowledge_ids.len() > 0);
        assert_eq!(broken.damage_vector.len(), 16);
    }

    #[test]
    fn test_kintsugi_repair() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "d"),
            KnowledgeItem::new("k2", "b", "d"),
            KnowledgeItem::new("k3", "c", "d"),
        ]);
        let state = AgentState::new("a", LifecyclePhase::Work)
            .with_knowledge(ks)
            .with_feature_dim(16);
        let broken = KintsugiAdjunction::break_state(&state, 0.5, 42);

        let golden = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("golden-1", "repair", "d"),
        ]);
        let repaired = KintsugiAdjunction::repair(&broken, &state, &golden);

        assert_eq!(repaired.phase, LifecyclePhase::Work);
        assert!(repaired.knowledge.len() > 0);
        assert!(repaired.id.contains("kintsugi"));
    }

    #[test]
    fn test_kintsugi_adjunction_verification() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "d"),
            KnowledgeItem::new("k2", "b", "d"),
        ]);
        let state = AgentState::new("a", LifecyclePhase::Work)
            .with_knowledge(ks)
            .with_feature_dim(16);
        let broken = KintsugiAdjunction::break_state(&state, 0.5, 42);
        let golden = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("g1", "patch", "d"),
        ]);
        assert!(KintsugiAdjunction::verify_adjunction(&broken, &state, &golden));
    }

    #[test]
    fn test_kintsugi_zero_damage() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "d"),
        ]);
        let state = AgentState::new("a", LifecyclePhase::Work)
            .with_knowledge(ks)
            .with_feature_dim(16);
        let broken = KintsugiAdjunction::break_state(&state, 0.0, 42);
        assert!(broken.lost_knowledge_ids.is_empty());
    }

    // ─── Generation Tracking Tests ──────────────────────────────────────

    #[test]
    fn test_generation_tracker_register() {
        let state = AgentState::new("a", LifecyclePhase::Birth).with_generation(0);
        let mut tracker = GenerationTracker::new();
        tracker.register(&state);
        assert_eq!(tracker.lineage_size(), 1);
        assert_eq!(tracker.max_generation, 0);
    }

    #[test]
    fn test_generation_tracker_mutation() {
        let state = AgentState::new("a", LifecyclePhase::Work);
        let mut tracker = GenerationTracker::new();
        tracker.register(&state);
        tracker.record_mutation("a");
        tracker.record_mutation("a");
        assert_eq!(tracker.lineage.get("a").unwrap().mutation_count, 2);
    }

    #[test]
    fn test_generation_diversity() {
        let states = vec![
            AgentState::new("a", LifecyclePhase::Work)
                .with_generation(0)
                .with_knowledge(KnowledgeSet::from_items(vec![
                    KnowledgeItem::new("k1", "a", "math"),
                ])),
            AgentState::new("b", LifecyclePhase::Work)
                .with_generation(1)
                .with_knowledge(KnowledgeSet::from_items(vec![
                    KnowledgeItem::new("k2", "b", "logic"),
                    KnowledgeItem::new("k3", "c", "math"),
                ])),
        ];
        let tracker = GenerationTracker::new();
        let div = tracker.diversity(&states);
        assert_eq!(*div.get(&0).unwrap(), 1);
        assert_eq!(*div.get(&1).unwrap(), 2);
    }

    #[test]
    fn test_generation_lineage() {
        let mut tracker = GenerationTracker::new();
        let parent = AgentState::new("p", LifecyclePhase::Work).with_generation(0);
        let child = AgentState::new("c", LifecyclePhase::Birth)
            .with_generation(1)
            .with_parents(vec!["p".to_string()]);
        tracker.register(&parent);
        tracker.register(&child);

        let lineage = tracker.lineage_of("c");
        assert!(lineage.contains(&"c".to_string()));
        assert!(lineage.contains(&"p".to_string()));
        assert!(tracker.is_descendant_of("c", "p"));
    }

    // ─── AgentCategory Tests ─────────────────────────────────────────────

    #[test]
    fn test_agent_category_creation() {
        let mut cat = AgentCategory::new();
        cat.add_state(AgentState::new("a", LifecyclePhase::Birth));
        cat.add_state(AgentState::new("b", LifecyclePhase::Work));
        assert_eq!(cat.objects.len(), 2);
    }

    #[test]
    fn test_agent_category_connected() {
        let mut cat = AgentCategory::new();
        cat.add_state(AgentState::new("a", LifecyclePhase::Work));
        cat.add_state(AgentState::new("b", LifecyclePhase::Work));
        cat.add_transition(Transition {
            source_id: "a".into(),
            target_id: "b".into(),
            knowledge_added: 1,
            knowledge_preserved: 0,
            label: "t".into(),
        });
        let pi0 = cat.connected_components();
        assert_eq!(pi0.count(), 1);
    }

    #[test]
    fn test_agent_category_conservation() {
        let mut cat = AgentCategory::new();
        let s = AgentState::new("a", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "d"),
            ]));
        cat.add_state(s);
        let cf = cat.apply_conservation();
        assert!(cf.discrete_cat.len() > 0);
    }

    #[test]
    fn test_agent_category_laws() {
        let cat = AgentCategory::new();
        assert!(cat.verify_composition());
        assert!(cat.verify_identities());
    }

    // ─── PLATO Lifecycle Tests ───────────────────────────────────────────

    #[test]
    fn test_plato_birth() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k0", "genesis", "core"),
        ]);
        let plato = PlatoLifecycle::birth("plato-1", ks, 0);
        assert_eq!(plato.current().phase, LifecyclePhase::Birth);
        assert_eq!(plato.current().knowledge.len(), 1);
    }

    #[test]
    fn test_plato_work_ticks() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k0", "genesis", "core"),
        ]);
        let mut plato = PlatoLifecycle::birth("plato-1", ks, 0);

        plato.work(
            vec![KnowledgeItem::new("k1", "learned", "math")],
            "tick-1",
        );
        plato.work(
            vec![KnowledgeItem::new("k2", "more", "logic")],
            "tick-2",
        );

        assert_eq!(plato.current().knowledge.len(), 3);
        assert_eq!(plato.diagram.length(), 3);
    }

    #[test]
    fn test_plato_sunset() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k0", "genesis", "core"),
        ]);
        let mut plato = PlatoLifecycle::birth("plato-1", ks, 0);
        plato.work(
            vec![KnowledgeItem::new("k1", "learned", "math")],
            "tick-1",
        );

        let result = plato.sunset();
        assert!(result.unique_knowledge_preserved);
        assert_eq!(result.source_count, 2);
    }

    #[test]
    fn test_plato_seed() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k0", "genesis", "core"),
        ]);
        let mut plato = PlatoLifecycle::birth("plato-1", ks, 0);
        plato.work(
            vec![KnowledgeItem::new("k1", "learned", "math")],
            "tick-1",
        );

        let seed = plato.seed();
        assert_eq!(seed.phase, LifecyclePhase::Seed);
        assert_eq!(seed.generation, 1);
        assert_eq!(seed.knowledge.len(), 2);
    }

    #[test]
    fn test_plato_full_lifecycle() {
        // Complete lifecycle: birth → work → work → sunset → seed → spawn
        let ks1 = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "math"),
        ]);
        let mut plato1 = PlatoLifecycle::birth("gen0-a", ks1, 0);
        plato1.work(vec![KnowledgeItem::new("k2", "b", "math")], "work");

        let ks2 = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k3", "c", "logic"),
        ]);
        let mut plato2 = PlatoLifecycle::birth("gen0-b", ks2, 0);
        plato2.work(vec![KnowledgeItem::new("k4", "d", "logic")], "work");

        // Sunset both
        let seed1 = plato1.seed();
        let seed2 = plato2.seed();

        // Spawn next generation
        let interface = SharedInterface::empty();
        let child = spawn(&seed1, &seed2, &interface, "gen1-child");

        assert_eq!(child.child.generation, 2);
        assert_eq!(child.child.knowledge.len(), 4);
    }

    // ─── Serialization Tests ─────────────────────────────────────────────

    #[test]
    fn test_knowledge_set_serialization() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "d"),
        ]);
        let json = serde_json::to_string(&ks).unwrap();
        let deserialized: KnowledgeSet = serde_json::from_str(&json).unwrap();
        assert_eq!(ks, deserialized);
    }

    #[test]
    fn test_agent_state_serialization() {
        let state = AgentState::new("a", LifecyclePhase::Work)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "d"),
            ]));
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: AgentState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.id, deserialized.id);
        assert_eq!(state.phase, deserialized.phase);
        assert_eq!(state.knowledge.len(), deserialized.knowledge.len());
    }

    #[test]
    fn test_connected_components_serialization() {
        let pi0 = ConnectedComponents::new();
        let json = serde_json::to_string(&pi0).unwrap();
        let _: ConnectedComponents = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_generation_tracker_serialization() {
        let tracker = GenerationTracker::new();
        let json = serde_json::to_string(&tracker).unwrap();
        let _: GenerationTracker = serde_json::from_str(&json).unwrap();
    }

    // ─── Integration / Edge Case Tests ───────────────────────────────────

    #[test]
    fn test_empty_knowledge_set() {
        let ks = KnowledgeSet::new();
        assert!(ks.is_empty());
        assert_eq!(ks.len(), 0);
        let v = ks.to_feature_vector(8);
        assert_eq!(v.iter().filter(|x| **x != 0.0).count(), 0);
    }

    #[test]
    fn test_shared_interface() {
        let mut ids = BTreeSet::new();
        ids.insert("k1".to_string());
        ids.insert("k2".to_string());
        let iface = SharedInterface::new("math", ids);
        assert_eq!(iface.len(), 2);
        assert!(!iface.is_empty());
    }

    #[test]
    fn test_conservation_signature_equality() {
        let sig1 = ConservationSignature {
            knowledge_count: 5,
            generation: 1,
            total_provenance: 3,
            phase: LifecyclePhase::Work,
            domains: 2,
        };
        let sig2 = ConservationSignature {
            knowledge_count: 5,
            generation: 1,
            total_provenance: 3,
            phase: LifecyclePhase::Work,
            domains: 2,
        };
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_broken_state_serialization() {
        let ks = KnowledgeSet::from_items(vec![
            KnowledgeItem::new("k1", "a", "d"),
        ]);
        let state = AgentState::new("a", LifecyclePhase::Work)
            .with_knowledge(ks)
            .with_feature_dim(8);
        let broken = KintsugiAdjunction::break_state(&state, 0.5, 42);
        let json = serde_json::to_string(&broken).unwrap();
        let _: BrokenState = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_diagram_default() {
        let diag = FilteredDiagram::default();
        assert_eq!(diag.length(), 0);
    }

    #[test]
    fn test_long_chain_colimit() {
        let s0 = AgentState::new("a:0", LifecyclePhase::Birth)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k0", "base", "d"),
            ]));
        let mut diag = FilteredDiagram::chain(s0);

        for i in 1..=10 {
            let mut ks = KnowledgeSet::new();
            for j in 0..=i {
                ks.insert(KnowledgeItem::new(&format!("k{}", j), &format!("v{}", j), "d"));
            }
            let s = AgentState::new(format!("a:{}", i), LifecyclePhase::Work)
                .with_knowledge(ks);
            diag.tick(s, format!("tick-{}", i));
        }

        assert_eq!(diag.length(), 11);
        let result = sunset(&diag);
        assert!(result.unique_knowledge_preserved);
        assert_eq!(result.knowledge_after_total, 11);
    }

    #[test]
    fn test_multi_generation_lineage() {
        let mut tracker = GenerationTracker::new();
        let g0 = AgentState::new("g0", LifecyclePhase::Birth).with_generation(0);
        let g1 = AgentState::new("g1", LifecyclePhase::Birth)
            .with_generation(1)
            .with_parents(vec!["g0".to_string()]);
        let g2 = AgentState::new("g2", LifecyclePhase::Birth)
            .with_generation(2)
            .with_parents(vec!["g1".to_string()]);

        tracker.register(&g0);
        tracker.register(&g1);
        tracker.register(&g2);

        assert!(tracker.is_descendant_of("g2", "g0"));
        assert!(!tracker.is_descendant_of("g0", "g2"));
        assert_eq!(tracker.max_generation, 2);
    }

    #[test]
    fn test_conservation_functor_default() {
        let cf = ConservationFunctor::default();
        assert!(cf.mapping.is_empty());
        assert!(cf.discrete_cat.is_empty());
    }

    #[test]
    fn test_agent_feature_vector_dimension() {
        let state = AgentState::new("a", LifecyclePhase::Work)
            .with_feature_dim(128)
            .with_knowledge(KnowledgeSet::from_items(vec![
                KnowledgeItem::new("k1", "a", "d"),
            ]));
        let v = state.feature_vector();
        assert_eq!(v.len(), 128);
    }
}
