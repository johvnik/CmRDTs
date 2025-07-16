use crate::actor::ActorId;
use serde::{Deserialize, Serialize};

/// A Dot represents a single event from an actor, identified by a sequence number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Dot {
    pub actor: ActorId,
    pub counter: u64,
}
