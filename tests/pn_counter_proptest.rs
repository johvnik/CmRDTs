use cmrdts::core::{ActorId, Replica};
use cmrdts::pn_counter::{Op, PNCounter};
use proptest::prelude::*;

// A strategy to generate a single random Op.
fn arb_op() -> impl Strategy<Value = Op> {
    prop_oneof![(1..50u64).prop_map(Op::Inc), (1..50u64).prop_map(Op::Dec),]
}

// A strategy to generate a vector of random operations.
fn arb_ops() -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec(arb_op(), 0..15)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]
    #[test]
    fn test_pn_counter_properties(
        ops_a in arb_ops(),
        ops_b in arb_ops(),
        ops_c in arb_ops()
    ) {
        // --- Arrange ---
        let mut replica_a = Replica::new(ActorId(1), PNCounter::default());
        for op in ops_a.clone() {
            replica_a.apply(op);
        }

        let mut replica_b = Replica::new(ActorId(2), PNCounter::default());
        for op in ops_b.clone() {
            replica_b.apply(op);
        }

        let mut replica_c = Replica::new(ActorId(3), PNCounter::default());
        for op in ops_c.clone() {
            replica_c.apply(op);
        }

        // --- Act & Assert ---

        // 1. Test Commutativity
        {
            let mut merged_ab = replica_a.clone();
            merged_ab.merge(replica_b.state().clone(), replica_b.clock().clone());

            let mut merged_ba = replica_b.clone();
            merged_ba.merge(replica_a.state().clone(), replica_a.clock().clone());

            prop_assert_eq!(merged_ab.state(), merged_ba.state(), "Commutativity failed");
        }


        // 2. Test Associativity
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

        // 3. Test Idempotence
        {
            let mut idempotent_a = replica_a.clone();
            idempotent_a.merge(replica_a.state().clone(), replica_a.clock().clone());
            prop_assert_eq!(idempotent_a.state(), replica_a.state(), "Idempotence failed");
        }

        // 4. Test Correctness of final value
        {
            let mut expected_sum: i64 = 0;
            for op in ops_a.iter().chain(ops_b.iter()).chain(ops_c.iter()) {
                match op {
                    Op::Inc(amount) => expected_sum += *amount as i64,
                    Op::Dec(amount) => expected_sum -= *amount as i64,
                }
            }

            let mut final_replica = replica_a.clone();
            final_replica.merge(replica_b.state().clone(), replica_b.clock().clone());
            final_replica.merge(replica_c.state().clone(), replica_c.clock().clone());

            prop_assert_eq!(final_replica.read(), expected_sum, "Final value calculation is incorrect");
        }
    }
}
