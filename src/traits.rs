use crate::ctx::AddCtx;

/// The core trait for all CmRDTs.
pub trait CmRDT {
    /// The operation type that can be applied to this CRDT.
    type Op;

    /// The value type that this CRDT represents.
    type Value;

    /// Apply an operation to the CRDT.
    fn apply(&mut self, op: Self::Op, ctx: AddCtx);

    /// Merge another CRDT replica into this one.
    fn merge(&mut self, other: Self);

    /// Read the current value of the CRDT.
    fn read(&self) -> Self::Value;
}
