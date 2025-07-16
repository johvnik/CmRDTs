use serde::{Deserialize, Serialize};

/// A unique identifier for a replica.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActorId(pub u64);
