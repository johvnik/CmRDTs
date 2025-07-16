use serde::{Deserialize, Serialize};

use crate::core::{Dot, VClock};

/// Context required for applying a new operation (the "add" context).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddCtx {
    pub dot: Dot,
    pub clock: VClock,
}

/// Context for reading a value (could be just the causal context).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadCtx {
    pub clock: VClock,
}
