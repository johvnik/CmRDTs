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
