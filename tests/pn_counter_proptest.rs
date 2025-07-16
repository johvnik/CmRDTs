use cmrdts::core::{ActorId, AddCtx, CmRDT, Dot, VClock};
use cmrdts::pn_counter::{Op, PNCounter};
use proptest::prelude::*;
use std::collections::BTreeMap;

// Proptest strategy to generate a vector of random operations.
fn arb_ops() -> impl Strategy<Value = Vec<(u64, bool, u64)>> {
    prop::collection::vec(
        (
            0..5u64,         // actor_id (small range to ensure conflicts)
            prop::bool::ANY, // true for Inc, false for Dec
            1..50u64,        // amount
        ),
        0..15, // Generate between 0 and 15 operations per replica
    )
}

// Helper function to apply a list of generated operations to a replica.
fn apply_ops(
    replica: &mut PNCounter,
    ops: &[(u64, bool, u64)],
    dot_counters: &mut BTreeMap<u64, u64>,
) {
    for &(actor, is_inc, amount) in ops {
        let op_counter = dot_counters.entry(actor).or_insert(0);
        *op_counter += 1;

        let op = if is_inc {
            Op::Inc(amount)
        } else {
            Op::Dec(amount)
        };
        let ctx = AddCtx {
            dot: Dot {
                actor: ActorId(actor),
                counter: *op_counter,
            },
            clock: VClock::default(),
        };
        replica.apply(op, ctx);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn test_pn_counter_properties(
        ops_a in arb_ops(),
        ops_b in arb_ops(),
        ops_c in arb_ops()
    ) {
        // --- Arrange ---
        let mut replica_a = PNCounter::default();
        let mut replica_b = PNCounter::default();
        let mut replica_c = PNCounter::default();

        let mut dot_counters = BTreeMap::new();

        apply_ops(&mut replica_a, &ops_a, &mut dot_counters);
        apply_ops(&mut replica_b, &ops_b, &mut dot_counters);
        apply_ops(&mut replica_c, &ops_c, &mut dot_counters);

        // --- Act & Assert ---

        // 1. Test Commutativity: a.merge(b) == b.merge(a)
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
            prop_assert_eq!(idempotent_a, replica_a.clone(), "Idempotence failed");
        }

        // 4. Test Correctness of final value
        {
            let mut expected_increments = BTreeMap::new();
            for (actor, count) in replica_a.increments.counters.iter()
                .chain(replica_b.increments.counters.iter())
                .chain(replica_c.increments.counters.iter())
            {
                let entry = expected_increments.entry(*actor).or_insert(0);
                *entry = (*entry).max(*count);
            }

            let mut expected_decrements = BTreeMap::new();
            for (actor, count) in replica_a.decrements.counters.iter()
                .chain(replica_b.decrements.counters.iter())
                .chain(replica_c.decrements.counters.iter())
            {
                let entry = expected_decrements.entry(*actor).or_insert(0);
                *entry = (*entry).max(*count);
            }

            let expected_sum = expected_increments.values().sum::<u64>() as i64 -
                               expected_decrements.values().sum::<u64>() as i64;

            // Merge all replicas together
            let mut final_replica = replica_a.clone();
            final_replica.merge(replica_b.clone());
            final_replica.merge(replica_c.clone());

            prop_assert_eq!(final_replica.read(), expected_sum, "Final value calculation is incorrect");
        }
    }
}
