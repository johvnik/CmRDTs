use crate::actor::ActorId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A Vector Clock tracks the state of all actors.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VClock(pub BTreeMap<ActorId, u64>);
