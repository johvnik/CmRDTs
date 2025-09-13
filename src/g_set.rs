use crate::core::{AddCtx, CmRDT};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt::Debug};

/// An operation-based, Grow-Only Set (CmRDT).
///
/// A GSet is a set where elements can only be added. Because adding the
/// same element twice is idempotent, we only need to store the unique
/// values themselves.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct GSet<T: Clone + Ord> {
    pub values: BTreeSet<T>,
}

/// The only operation for a GSet is to add a value.
#[derive(Debug, Clone)]
pub enum Op<T> {
    Add(T),
}

impl<T: Clone + Ord> CmRDT for GSet<T> {
    type Op = Op<T>;
    type Value = BTreeSet<T>;

    fn apply(&mut self, op: Self::Op, _ctx: AddCtx) {
        let Op::Add(value) = op;

        self.values.insert(value);
    }

    fn merge(&mut self, other: Self) {
        self.values.extend(other.values);
    }

    fn read(&self) -> Self::Value {
        self.values.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ActorId, Replica};
    use std::collections::BTreeSet;

    #[test]
    fn test_initial_value_is_empty() {
        let replica = Replica::new(ActorId(1), GSet::<i32>::default());
        assert!(replica.read().is_empty());
    }

    #[test]
    fn test_add_and_read() {
        let mut replica = Replica::new(ActorId(1), GSet::default());
        replica.apply(Op::Add("hello".to_string()));
        replica.apply(Op::Add("world".to_string()));

        let mut expected = BTreeSet::new();
        expected.insert("hello".to_string());
        expected.insert("world".to_string());
        assert_eq!(replica.read(), expected);
    }

    #[test]
    fn test_add_is_idempotent() {
        let mut replica = Replica::new(ActorId(1), GSet::default());
        replica.apply(Op::Add(123));
        replica.apply(Op::Add(123));
        replica.apply(Op::Add(123));

        let mut expected = BTreeSet::new();
        expected.insert(123);
        assert_eq!(replica.read(), expected);
        assert_eq!(replica.read().len(), 1);
    }

    #[test]
    fn test_merge_takes_union() {
        let mut replica_a = Replica::new(ActorId(1), GSet::default());
        replica_a.apply(Op::Add(1));
        replica_a.apply(Op::Add(2));

        let mut replica_b = Replica::new(ActorId(2), GSet::default());
        replica_b.apply(Op::Add(2));

        replica_a.merge(replica_b.state().clone(), replica_b.clock().clone());

        let mut expected = BTreeSet::new();
        expected.insert(1);
        expected.insert(2);
        assert_eq!(replica_a.read(), expected);
    }
}
