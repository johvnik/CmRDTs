use crate::{
    Dot,
    core::{AddCtx, CmRDT},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// An operation-based, Last-Write-Wins Register (CmRDT).
///
/// LWWRegister is a simple key-value store where concurrent updates are
/// resolved by picking the one with the highest timestamp. In this system,
/// the `Dot` (acting as a Hybrid Logical Clock timestamp) serves as the timestamp.
/// This implementation uses `Option` to correctly model the initial state where
/// no value has been set yet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct LWWRegister<T: Clone> {
    /// The current value of the register, if one has been set.
    pub value: Option<T>,
    /// The dot of the operation that set the current value.
    pub dot: Option<Dot>,
}

/// The only operation for a LWWRegister is to set its value.
#[derive(Debug, Clone)]
pub enum Op<T> {
    Set(T),
}

impl<T: Clone + Debug + PartialEq> CmRDT for LWWRegister<T> {
    type Op = Op<T>;
    type Value = Option<T>;

    /// Applies a `Set` operation if its dot is causally newer than the current dot.
    fn apply(&mut self, op: Self::Op, ctx: AddCtx) {
        let Op::Set(value) = op;

        match self.dot {
            Some(current_dot) if current_dot >= ctx.dot => {
                // Current dot is newer or the same, so we ignore the op.
            }
            // The new dot is strictly greater OR we have no dot yet.
            _ => {
                self.value = Some(value);
                self.dot = Some(ctx.dot);
            }
        }
    }

    /// Merges another LWWRegister into this one, keeping the value with the greater dot.
    fn merge(&mut self, other: Self) {
        match (self.dot, other.dot) {
            (_, None) => {
                // The other register is empty, so we have nothing to do.
            }
            (None, Some(_)) => {
                // We are empty, so we become a clone of the other register.
                *self = other;
            }
            (Some(self_dot), Some(other_dot)) if other_dot > self_dot => {
                // The other's dot is greater, so we become a clone of it.
                *self = other;
            }
            _ => {
                // Our dot is greater or equal, so we have nothing to do.
            }
        }
    }

    /// Reads the current value of the register.
    fn read(&self) -> Self::Value {
        self.value.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ActorId, Replica, VClock};

    #[test]
    fn test_initial_value_is_none() {
        let replica = Replica::new(ActorId(1), LWWRegister::<String>::default());
        assert_eq!(replica.read(), None);
    }

    #[test]
    fn test_apply_and_read() {
        let mut replica = Replica::new(ActorId(1), LWWRegister::default());
        replica.apply(Op::Set("hello".to_string()));
        assert_eq!(replica.read(), Some("hello".to_string()));
    }

    #[test]
    fn test_apply_newer_dot_wins() {
        let mut replica = Replica::new(ActorId(1), LWWRegister::default());
        replica.apply(Op::Set("first".to_string()));
        replica.apply(Op::Set("second".to_string()));
        assert_eq!(replica.read(), Some("second".to_string()));
    }

    #[test]
    fn test_apply_older_dot_is_ignored() {
        let mut replica = Replica::new(ActorId(2), LWWRegister::default());

        // 1. Manually apply a remote op with a "future" dot to establish a baseline.
        //    Crucially, we must also provide the VClock that the sender would have had.
        let mut remote_clock = VClock::default();
        remote_clock.0.insert(ActorId(1), 100);
        let remote_ctx = AddCtx {
            dot: Dot {
                actor: ActorId(1),
                counter: 100,
            },
            clock: remote_clock,
        };
        replica.apply_remote(Op::Set("future".to_string()), remote_ctx);

        // We expect the value to be "future"
        assert_eq!(replica.read(), Some("future".to_string()));

        // 2. Now, create another remote op from a different actor that is causally "older"
        //    because its dot's counter is smaller.
        let older_remote_ctx = AddCtx {
            dot: Dot {
                actor: ActorId(3),
                counter: 99,
            },
            clock: Default::default(), // This clock doesn't matter for this part of the test
        };

        // 3. Apply the older operation.
        replica.apply_remote(Op::Set("past".to_string()), older_remote_ctx);

        // 4. Assert that the state did NOT change, because the older dot was ignored.
        assert_eq!(replica.read(), Some("future".to_string()));
    }

    #[test]
    fn test_apply_tie_break_with_actor_id() {
        let mut replica = Replica::new(ActorId(2), LWWRegister::default());

        // Apply a remote op from a "smaller" actor with a high counter.
        let remote_ctx = AddCtx {
            dot: Dot {
                actor: ActorId(1),
                counter: 100,
            },
            clock: Default::default(),
        };
        replica.apply_remote(Op::Set("from_actor_1".to_string()), remote_ctx);

        // Manually construct a remote op from a "larger" actor with the same counter.
        let remote_ctx_2 = AddCtx {
            dot: Dot {
                actor: ActorId(3),
                counter: 100,
            },
            clock: Default::default(),
        };
        replica.apply_remote(Op::Set("from_actor_3".to_string()), remote_ctx_2);

        // Actor 3 should win the tie-break.
        assert_eq!(replica.read(), Some("from_actor_3".to_string()));
    }

    #[test]
    fn test_merge_newer_dot_wins() {
        let mut replica_a = Replica::new(ActorId(1), LWWRegister::default());
        replica_a.apply(Op::Set("A".to_string())); // dot {a:1, c:1}

        let mut replica_b = Replica::new(ActorId(1), LWWRegister::default());
        replica_b.apply(Op::Set("B".to_string())); // dot {a:1, c:1}
        replica_b.apply(Op::Set("C".to_string())); // dot {a:1, c:2}

        replica_a.merge(replica_b.state().clone(), replica_b.clock().clone());
        assert_eq!(replica_a.read(), Some("C".to_string()));
    }

    #[test]
    fn test_merge_is_idempotent() {
        let mut replica_a = Replica::new(ActorId(1), LWWRegister::default());
        replica_a.apply(Op::Set("A".to_string()));

        let mut replica_b = Replica::new(ActorId(2), LWWRegister::default());
        replica_b.apply(Op::Set("B".to_string()));

        replica_a.merge(replica_b.state().clone(), replica_b.clock().clone());
        let expected_read = replica_a.read();

        // A second merge should not change the state.
        replica_a.merge(replica_b.state().clone(), replica_b.clock().clone());
        assert_eq!(replica_a.read(), expected_read);
    }

    #[test]
    fn test_causality_actor_b_wins_after_syncing() {
        // Arrange: Create two replicas
        let mut replica_a = Replica::new(ActorId(1), LWWRegister::<String>::default());
        let mut replica_b = Replica::new(ActorId(2), LWWRegister::<String>::default());

        let mut a_ops = Vec::new();

        // Act 1: Actor A performs 10 operations in a row.
        for i in 0..10 {
            let op_val = format!("A_{}", i);
            let (op, ctx) = replica_a.apply(Op::Set(op_val));
            a_ops.push((op, ctx));
        }

        // Sanity check: Replica A's value should be the last one it set.
        // Its dot's counter will be 10.
        assert_eq!(replica_a.read(), Some("A_9".to_string()));
        assert_eq!(replica_a.state().dot.unwrap().counter, 10);

        // Act 2: Actor B receives all of Actor A's operations.
        for (op, ctx) in a_ops {
            replica_b.apply_remote(op, ctx);
        }

        // Sanity check: Replica B should now be in sync with A.
        // Its VClock should know that the latest time is 10.
        assert_eq!(replica_b.read(), Some("A_9".to_string()));
        assert_eq!(replica_b.clock().max_counter(), 10);

        // Act 3: Actor B, having seen all of A's work, performs a single new operation.
        replica_b.apply(Op::Set("B_wins".to_string()));

        // Assert: Actor B's new write must win.
        // Its HLC logic should generate a dot with a counter of 11 (max_counter + 1),
        // which is greater than A's dot with a counter of 10.
        assert_eq!(replica_b.read(), Some("B_wins".to_string()));

        // We also inspect the internal dot to be certain.
        let final_dot = replica_b.state().dot.unwrap();
        assert_eq!(final_dot.actor, ActorId(2));
        assert_eq!(final_dot.counter, 11);
    }
}
