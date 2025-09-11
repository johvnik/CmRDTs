use crate::{
    Dot,
    core::{AddCtx, CmRDT},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An operation-based, last-write-wins register.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LWWRegister<T>
where
    T: Clone + PartialEq,
{
    pub ops: BTreeMap<Dot, T>,
}
