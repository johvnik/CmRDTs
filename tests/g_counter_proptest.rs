use cmrdts::core::{ActorId, Replica};
use cmrdts::g_counter::{GCounter, Op};
use proptest::prelude::*;

// Proptest strategy to generate a vector of random increment amounts.
// We don't need to generate actor IDs here, as the test setup will assign them.
fn arb_ops() -> impl Strategy<Value = Vec<u64>> {
    prop::collection::vec(
        1..100u64, // increment_amount
        0..15,     // Generate between 0 and 15 operations per replica
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
        // Create three distinct replicas, each with its own actor ID.
        let mut replica_a = Replica::new(ActorId(1), GCounter::default());
        for amount in ops_a {
            replica_a.apply(Op::Inc(amount));
        }

        let mut replica_b = Replica::new(ActorId(2), GCounter::default());
        for amount in ops_b {
            replica_b.apply(Op::Inc(amount));
        }

        let mut replica_c = Replica::new(ActorId(3), GCounter::default());
        for amount in ops_c {
            replica_c.apply(Op::Inc(amount));
        }


        // --- Act & Assert ---

        // 1. Test Commutativity: a.merge(b) == b.merge(a)
        {
            let mut merged_ab = replica_a.clone();
            merged_ab.merge(replica_b.state().clone(), replica_b.clock().clone());

            let mut merged_ba = replica_b.clone();
            merged_ba.merge(replica_a.state().clone(), replica_a.clock().clone());

            // We only need to assert the CRDT state, as clock state is an implementation detail
            // of the replica, not a property of the CRDT itself.
            prop_assert_eq!(merged_ab.state(), merged_ba.state(), "Commutativity failed");
        }


        // 2. Test Associativity: (a.merge(b)).merge(c) == a.merge(b.merge(c))
        {
            let mut merged_ab = replica_a.clone();
            merged_ab.merge(replica_b.state().clone(), replica_b.clock().clone());
            let mut merged_ab_c = merged_ab;
            merged_ab_c.merge(replica_c.state().clone(), replica_c.clock().clone());


            let mut merged_bc = replica_b.clone();
            merged_bc.merge(replica_c.state().clone(), replica_c.clock().clone());
            let mut merged_a_bc = replica_a.clone();
            merged_a_bc.merge(merged_bc.state().clone(), merged_bc.clock().clone());

            prop_assert_eq!(merged_ab_c.state(), merged_a_bc.state(), "Associativity failed");
        }

        // 3. Test Idempotence: a.merge(a) == a
        {
            let mut idempotent_a = replica_a.clone();
            idempotent_a.merge(replica_a.state().clone(), replica_a.clock().clone());
            prop_assert_eq!(idempotent_a.state(), replica_a.state(), "Idempotence failed");
        }
    }
}
