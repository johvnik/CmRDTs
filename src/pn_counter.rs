use crate::core::{AddCtx, CmRDT};
use crate::g_counter::{self, GCounter};
use serde::{Deserialize, Serialize};

/// A Positive-Negative Counter.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PNCounter {
    pub increments: GCounter,
    pub decrements: GCounter,
}

/// Operations for a PNCounter can be increments or decrements.
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
    use crate::core::{ActorId, Dot, VClock};

    fn actor_ctx(id: u64, counter: u64) -> AddCtx {
        AddCtx {
            dot: Dot {
                actor: ActorId(id),
                counter,
            },
            clock: VClock::default(),
        }
    }

    #[test]
    fn test_apply_inc_and_dec() {
        // Arrange
        let mut counter = PNCounter::default();

        // Act
        counter.apply(Op::Inc(10), actor_ctx(1, 1));
        counter.apply(Op::Dec(3), actor_ctx(1, 2));
        counter.apply(Op::Inc(5), actor_ctx(1, 3));

        // Assert
        assert_eq!(counter.read(), 12); // (10 + 5) - 3
    }

    #[test]
    fn test_read_value_can_be_negative() {
        // Arrange
        let mut counter = PNCounter::default();

        // Act
        counter.apply(Op::Inc(5), actor_ctx(1, 1));
        counter.apply(Op::Dec(10), actor_ctx(1, 2));

        // Assert
        assert_eq!(counter.read(), -5);
    }

    #[test]
    fn test_merge_with_separate_replicas() {
        // Arrange
        let mut replica_a = PNCounter::default();
        let mut replica_b = PNCounter::default();

        // Act: replica_a gets increments, replica_b gets decrements
        replica_a.apply(Op::Inc(10), actor_ctx(1, 1));
        replica_b.apply(Op::Dec(4), actor_ctx(2, 1));

        // Merge replica_b into replica_a
        replica_a.merge(replica_b);

        // Assert: the final value reflects both operations
        assert_eq!(replica_a.read(), 6); // 10 - 4
    }

    #[test]
    fn test_merge_is_commutative() {
        // Arrange
        let mut replica_a = PNCounter::default();
        replica_a.apply(Op::Inc(10), actor_ctx(1, 1));
        replica_a.apply(Op::Dec(2), actor_ctx(1, 2)); // Final value: 8

        let mut replica_b = PNCounter::default();
        replica_b.apply(Op::Inc(5), actor_ctx(2, 1));
        replica_b.apply(Op::Dec(8), actor_ctx(2, 2)); // Final value: -3

        let mut merged_ab = replica_a.clone();
        let mut merged_ba = replica_b.clone();

        // Act
        merged_ab.merge(replica_b); // Merge B into A
        merged_ba.merge(replica_a); // Merge A into B

        // Assert: both should converge to the same value
        assert_eq!(merged_ab.read(), 5); // (10 + 5) - (2 + 8) = 5
        assert_eq!(merged_ba.read(), 5);
        assert_eq!(merged_ab, merged_ba);
    }
}
