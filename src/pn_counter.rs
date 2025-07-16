use crate::ctx::AddCtx;
use crate::gcounter::{self, GCounter};
use crate::traits::CmRDT;
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
            Op::Inc(amount) => self.increments.apply(gcounter::Op::Inc(amount), ctx),
            Op::Dec(amount) => self.decrements.apply(gcounter::Op::Inc(amount), ctx),
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
