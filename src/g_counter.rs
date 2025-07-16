use crate::{
    Dot,
    core::{AddCtx, CmRDT},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An operation-based, grow-only counter (CmRDT).
///
/// Unlike a state-based G-Counter (CvRDT) which merges actor totals using `max()`,
/// this implementation stores a log of every individual `Inc` operation. Each
/// operation is uniquely identified by its `Dot`, ensuring that all increments
/// are preserved when replica logs are merged.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GCounter {
    pub ops: BTreeMap<Dot, u64>,
}

/// The only operation for a GCounter is to increment its value.
#[derive(Debug, Clone)]
pub enum Op {
    Inc(u64),
}

impl CmRDT for GCounter {
    type Op = Op;
    type Value = u64;

    /// Records an operation, identified by its dot.
    fn apply(&mut self, op: Self::Op, ctx: AddCtx) {
        match op {
            Op::Inc(amount) => {
                self.ops.insert(ctx.dot, amount);
            }
        }
    }

    fn merge(&mut self, other: Self) {
        // `extend` will overwrite our ops with the other's if the keys (Dots) are the same,
        // which is fine since the operation (dot -> amount) is identical.
        self.ops.extend(other.ops);
    }

    fn read(&self) -> Self::Value {
        self.ops.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ActorId, Replica};

    #[test]
    fn test_initial_value_is_zero() {
        // Arrange
        let replica = Replica::new(ActorId(1), GCounter::default());

        // Assert
        assert_eq!(replica.read(), 0);
    }

    #[test]
    fn test_apply_and_read() {
        // Arrange
        let mut replica = Replica::new(ActorId(1), GCounter::default());

        // Act
        replica.apply(Op::Inc(5));
        replica.apply(Op::Inc(10));

        // Assert
        assert_eq!(replica.read(), 15);
    }

    #[test]
    fn test_merge_with_another_counter() {
        // Arrange
        let mut replica_a = Replica::new(ActorId(1), GCounter::default());
        let mut replica_b = Replica::new(ActorId(2), GCounter::default());

        // Act: different actors increment on different replicas
        replica_a.apply(Op::Inc(5));
        replica_b.apply(Op::Inc(10));

        // Merge replica_b into replica_a
        replica_a.merge(replica_b.state().clone(), replica_b.clock().clone());

        // Assert: the final value is the sum of all actors
        assert_eq!(replica_a.read(), 15);
    }

    #[test]
    fn test_op_based_logic_preserves_distinct_ops() {
        // Arrange
        let mut replica = Replica::new(ActorId(1), GCounter::default());

        // Act: Apply two distinct operations from the same actor
        replica.apply(Op::Inc(10)); // Generates Dot { actor: 1, counter: 1 }
        replica.apply(Op::Inc(5)); // Generates Dot { actor: 1, counter: 2 }
        replica.apply(Op::Inc(3)); // Generates Dot { actor: 1, counter: 2 }

        // Assert: The internal state should contain two separate operations,
        // and the read value should be their sum.
        assert_eq!(replica.state().ops.len(), 3);
        assert_eq!(replica.read(), 18);
    }

    #[test]
    fn test_merge_is_idempotent() {
        // Arrange
        let mut replica_a = Replica::new(ActorId(1), GCounter::default());
        let mut replica_b = Replica::new(ActorId(2), GCounter::default());

        replica_a.apply(Op::Inc(5));
        replica_b.apply(Op::Inc(10));

        replica_a.merge(replica_b.state().clone(), replica_b.clock().clone());

        let state_before_merge = replica_a.state().clone();
        let clock_before_merge = replica_a.clock().clone();
        let value_before_merge = replica_a.read();

        // Act: merge the replica with its own, identical state.
        replica_a.merge(state_before_merge, clock_before_merge);

        // Assert: the value should not change
        assert_eq!(replica_a.read(), value_before_merge);
    }
}
