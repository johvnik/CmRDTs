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
