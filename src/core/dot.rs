use serde::{Deserialize, Serialize};

use crate::core::ActorId;

/// A Dot represents a single event from an actor, identified by a sequence number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Dot {
    pub actor: ActorId,
    pub counter: u64,
}
