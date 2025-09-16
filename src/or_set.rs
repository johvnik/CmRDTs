use crate::{
    Dot,
    core::{AddCtx, CmRDT},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Debug,
    hash::Hash,
};

/// An operation-based, Observed-Remove Set (CmRDT).
///
/// An ORSet is a set that allows both additions and removals. It achieves this
/// by assigning a unique tag (a `Dot`) to each addition. A removal then acts
/// on all seen additions of a given element. This gives it an "add-wins"
/// bias in the case of concurrent add/remove operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct ORSet<T: Ord + Clone> {
    /// A map of members to a set of the unique dots for each add operation.
    adds: BTreeMap<T, BTreeSet<Dot>>,
    /// The dots of removed members.
    removes: BTreeSet<Dot>,
}

#[derive(Debug, Clone)]
pub enum Op<T> {
    Add(T),
    Remove(T),
}

impl<T: Clone + Ord> CmRDT for ORSet<T> {
    type Op = Op<T>;
    type Value = BTreeSet<T>;

    fn apply(&mut self, op: Self::Op, ctx: AddCtx) {
        match op {
            Op::Add(value) => {
                // Insert the value with its unique dot into the adds map.
                self.adds.entry(value).or_default().insert(ctx.dot);
            }
            Op::Remove(value) => {
                // A remove op should only apply to dots that the replica has observed.
                // We find all the dots associated with this value in our adds map
                // and add them to the removes set.
                if let Some(dots) = self.adds.get(&value) {
                    for dot in dots {
                        // Only remove the dot if the actor initiating the remove op
                        // has seen it (i.e., it's in their VClock).
                        if ctx.clock.contains(dot) {
                            self.removes.insert(*dot);
                        }
                    }
                }
            }
        }
    }

    /// Merges another ORSet into this one by taking the union of their adds and removes.
    fn merge(&mut self, other: Self) {
        for (value, dots) in other.adds {
            self.adds.entry(value).or_default().extend(dots);
        }
        self.removes.extend(other.removes);
    }

    /// The value of the set is the set of keys in the adds map that still have
    /// at least one dot that has not been removed.
    fn read(&self) -> Self::Value {
        self.adds
            .iter()
            .filter(|(_value, dots)| {
                // An element is present if any of its add-dots are not in the remove set.
                !dots.is_subset(&self.removes)
            })
            .map(|(value, _dots)| value.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ActorId, Replica};

    #[test]
    fn test_add_and_read() {
        let mut replica = Replica::new(ActorId(1), ORSet::default());
        replica.apply(Op::Add("hello".to_string()));
        let mut expected = BTreeSet::new();
        expected.insert("hello".to_string());
        assert_eq!(replica.read(), expected);
    }

    #[test]
    fn test_add_and_remove() {
        let mut replica = Replica::new(ActorId(1), ORSet::default());
        replica.apply(Op::Add(123));
        assert!(!replica.read().is_empty());
        replica.apply(Op::Remove(123));
        assert!(replica.read().is_empty());
    }

    #[test]
    fn test_concurrent_add_and_remove_add_wins() {
        let mut replica_a = Replica::new(ActorId(1), ORSet::default());
        let mut replica_b = Replica::new(ActorId(2), ORSet::default());

        // Both start with {10} in the set.
        let (op, ctx) = replica_a.apply(Op::Add(10));
        replica_b.apply_remote(op.clone(), ctx.clone());
        assert!(replica_a.read().contains(&10));
        assert!(replica_b.read().contains(&10));

        // Concurrently, A removes 10, B adds 10 again.
        let (remove_op_a, remove_ctx_a) = replica_a.apply(Op::Remove(10));
        let (add_op_b, add_ctx_b) = replica_b.apply(Op::Add(10));

        // B receives A's remove op.
        replica_b.apply_remote(remove_op_a, remove_ctx_a);

        // A receives B's add op.
        replica_a.apply_remote(add_op_b, add_ctx_b);

        // After syncing, both replicas should have 10 in the set.
        // B's add op had a dot that A's remove op could not have seen, so it "wins".
        assert!(replica_a.read().contains(&10));
        assert!(replica_b.read().contains(&10));
    }

    #[test]
    fn test_merge() {
        let mut replica_a = Replica::new(ActorId(1), ORSet::default());
        replica_a.apply(Op::Add("A"));
        replica_a.apply(Op::Add("B"));
        replica_a.apply(Op::Remove("A"));

        let mut replica_b = Replica::new(ActorId(2), ORSet::default());
        replica_b.apply(Op::Add("B"));
        replica_b.apply(Op::Add("C"));

        replica_a.merge(replica_b.state().clone(), replica_b.clock().clone());

        let mut expected = BTreeSet::new();
        expected.insert("B");
        expected.insert("C");
        assert_eq!(replica_a.read(), expected);
    }
}
