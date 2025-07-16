use cmrdts::core::{ActorId, AddCtx, CmRDT, Dot, VClock};
use cmrdts::g_counter::{GCounter, Op};
use proptest::prelude::*;

// Helper to create a simple context for a given actor.
fn ctx_for(actor: u64) -> AddCtx {
    AddCtx {
        dot: Dot {
            actor: ActorId(actor),
            counter: 1,
        },
        clock: VClock::default(),
    }
}

// Proptest strategy to generate a vector of random operations.
fn arb_ops() -> impl Strategy<Value = Vec<(u64, u64)>> {
    prop::collection::vec(
        (0..10u64, 1..100u64), // (actor_id, increment_amount)
        0..20,                 // Generate between 0 and 20 operations
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]
    #[test]
    fn test_g_counter_properties(
        ops_a in arb_ops(),
        ops_b in arb_ops(),
        ops_c in arb_ops()
    ) {
        // --- Arrange ---
        let mut replica_a = GCounter::default();
        for (actor, amount) in ops_a {
            replica_a.apply(Op::Inc(amount), ctx_for(actor));
        }

        let mut replica_b = GCounter::default();
        for (actor, amount) in ops_b {
            replica_b.apply(Op::Inc(amount), ctx_for(actor));
        }

        let mut replica_c = GCounter::default();
        for (actor, amount) in ops_c {
            replica_c.apply(Op::Inc(amount), ctx_for(actor));
        }

        // --- Act & Assert ---

        // 1. Test Commutativity: a.merge(b) == b.merge(a)
        // Each test now starts with fresh clones to avoid ownership issues.
        {
            let mut merged_ab = replica_a.clone();
            merged_ab.merge(replica_b.clone());

            let mut merged_ba = replica_b.clone();
            merged_ba.merge(replica_a.clone());

            prop_assert_eq!(merged_ab, merged_ba, "Commutativity failed");
        }


        // 2. Test Associativity: (a.merge(b)).merge(c) == a.merge(b.merge(c))
        {
            let mut merged_ab = replica_a.clone();
            merged_ab.merge(replica_b.clone());
            let mut merged_ab_c = merged_ab;
            merged_ab_c.merge(replica_c.clone());


            let mut merged_bc = replica_b.clone();
            merged_bc.merge(replica_c.clone());
            let mut merged_a_bc = replica_a.clone();
            merged_a_bc.merge(merged_bc);

            prop_assert_eq!(merged_ab_c, merged_a_bc, "Associativity failed");
        }

        // 3. Test Idempotence: a.merge(a) == a
        {
            let mut idempotent_a = replica_a.clone();
            idempotent_a.merge(replica_a.clone());
            prop_assert_eq!(idempotent_a, replica_a, "Idempotence failed");
        }
    }
}
