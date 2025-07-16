use crate::core::{ActorId, AddCtx, CmRDT};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A Grow-Only Counter.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GCounter {
    pub counters: BTreeMap<ActorId, u64>,
}

/// The only operation for a GCounter is to increment its value.
pub enum Op {
    Inc(u64),
}

impl CmRDT for GCounter {
    type Op = Op;
    type Value = u64;

    fn apply(&mut self, op: Self::Op, ctx: AddCtx) {
        // A GCounter doesn't use the full AddCtx, it only cares about the actor
        let actor_id = ctx.dot.actor;
        match op {
            Op::Inc(amount) => {
                let entry = self.counters.entry(actor_id).or_insert(0);
                *entry += amount;
            }
        }
    }

    fn merge(&mut self, other: Self) {
        for (actor, other_count) in other.counters {
            let self_count = self.counters.entry(actor).or_insert(0);
            *self_count = (*self_count).max(other_count);
        }
    }

    fn read(&self) -> Self::Value {
        self.counters.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Dot, VClock};

    fn actor_ctx(id: u64) -> AddCtx {
        AddCtx {
            dot: Dot {
                actor: ActorId(id),
                counter: 1, // Counter can be arbitrary for GCounter tests
            },
            clock: VClock::default(),
        }
    }

    #[test]
    fn test_initial_value_is_zero() {
        let counter = GCounter::default();
        assert_eq!(counter.read(), 0);
    }

    #[test]
    fn test_apply_and_read() {
        // Arrange
        let mut counter = GCounter::default();

        // Act
        counter.apply(Op::Inc(5), actor_ctx(1));
        counter.apply(Op::Inc(10), actor_ctx(1));

        // Assert
        assert_eq!(counter.read(), 15);
    }

    #[test]
    fn test_merge_with_another_counter() {
        // Arrange
        let mut replica_a = GCounter::default();
        let mut replica_b = GCounter::default();

        // Act: different actors increment on different replicas
        replica_a.apply(Op::Inc(5), actor_ctx(1));
        replica_b.apply(Op::Inc(10), actor_ctx(2));

        // Merge replica_b into replica_a
        replica_a.merge(replica_b);

        // Assert: the final value is the sum of all actors
        assert_eq!(replica_a.read(), 15);
    }

    #[test]
    fn test_merge_chooses_the_max_value_for_an_actor() {
        // Arrange
        let mut replica_a = GCounter::default();
        replica_a.apply(Op::Inc(10), actor_ctx(1)); // Actor 1 has 10

        let mut replica_b = GCounter::default();
        replica_b.apply(Op::Inc(5), actor_ctx(1)); // Actor 1 has 5

        // Act: merge b into a. The value for Actor 1 should become max(10, 5) = 10.
        replica_a.merge(replica_b);

        // Assert
        assert_eq!(replica_a.read(), 10);
    }

    #[test]
    fn test_merge_is_idempotent() {
        // Arrange
        let mut replica_a = GCounter::default();
        replica_a.apply(Op::Inc(5), actor_ctx(1));
        replica_a.apply(Op::Inc(10), actor_ctx(2));

        let replica_b = replica_a.clone(); // create an identical replica

        // Act: merge the identical replica into the original
        replica_a.merge(replica_b);

        // Assert: the value should not change
        assert_eq!(replica_a.read(), 15);
    }
}
