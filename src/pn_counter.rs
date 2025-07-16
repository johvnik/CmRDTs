use crate::core::{AddCtx, CmRDT};
use crate::g_counter::{self, GCounter};
use serde::{Deserialize, Serialize};

/// A Positive-Negative Counter, implemented as a composition of two op-based G-Counters.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PNCounter {
    pub increments: GCounter,
    pub decrements: GCounter,
}

/// Operations for a PNCounter can be increments or decrements.
#[derive(Debug, Clone, Copy)]
pub enum Op {
    Inc(u64),
    Dec(u64),
}

impl CmRDT for PNCounter {
    type Op = Op;
    type Value = i64;

    fn apply(&mut self, op: Self::Op, ctx: AddCtx) {
        match op {
            Op::Inc(amount) => self.increments.apply(g_counter::Op::Inc(amount), ctx),
            Op::Dec(amount) => self.decrements.apply(g_counter::Op::Inc(amount), ctx),
        }
    }

    fn merge(&mut self, other: Self) {
        self.increments.merge(other.increments);
        self.decrements.merge(other.decrements);
    }

    fn read(&self) -> Self::Value {
        self.increments.read() as i64 - self.decrements.read() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ActorId, Replica};

    #[test]
    fn test_apply_inc_and_dec() {
        // Arrange
        let mut replica = Replica::new(ActorId(1), PNCounter::default());

        // Act
        replica.apply(Op::Inc(10));
        replica.apply(Op::Dec(3));
        replica.apply(Op::Inc(5));

        // Assert
        assert_eq!(replica.read(), 12); // (10 + 5) - 3
    }

    #[test]
    fn test_read_value_can_be_negative() {
        // Arrange
        let mut replica = Replica::new(ActorId(1), PNCounter::default());

        // Act
        replica.apply(Op::Inc(5));
        replica.apply(Op::Dec(10));

        // Assert
        assert_eq!(replica.read(), -5);
    }

    #[test]
    fn test_merge_with_separate_replicas() {
        // Arrange
        let mut replica_a = Replica::new(ActorId(1), PNCounter::default());
        let mut replica_b = Replica::new(ActorId(2), PNCounter::default());

        // Act: replica_a gets increments, replica_b gets decrements
        replica_a.apply(Op::Inc(10));
        replica_b.apply(Op::Dec(4));

        // Merge replica_b's state and clock into replica_a
        replica_a.merge(replica_b.state().clone(), replica_b.clock().clone());

        // Assert: the final value reflects both operations
        assert_eq!(replica_a.read(), 6); // 10 - 4
    }

    #[test]
    fn test_merge_is_commutative() {
        // Arrange
        let mut replica_a = Replica::new(ActorId(1), PNCounter::default());
        replica_a.apply(Op::Inc(10));
        replica_a.apply(Op::Dec(2)); // Final value: 8

        let mut replica_b = Replica::new(ActorId(2), PNCounter::default());
        replica_b.apply(Op::Inc(5));
        replica_b.apply(Op::Dec(8)); // Final value: -3

        let mut merged_ab = replica_a.clone();
        merged_ab.merge(replica_b.state().clone(), replica_b.clock().clone());

        let mut merged_ba = replica_b.clone();
        merged_ba.merge(replica_a.state().clone(), replica_a.clock().clone());

        // Assert: both should converge to the same value and state
        assert_eq!(merged_ab.read(), 5); // (10 + 5) - (2 + 8) = 5
        assert_eq!(merged_ba.read(), 5);
        assert_eq!(merged_ab.state(), merged_ba.state());
    }
}
