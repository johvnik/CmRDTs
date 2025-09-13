use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::core::ActorId;

/// A Vector Clock tracks the state of all actors.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VClock(pub BTreeMap<ActorId, u64>);

impl VClock {
    /// Returns the highest counter value in the clock for any actor.
    pub fn max_counter(&self) -> u64 {
        self.0.values().max().cloned().unwrap_or(0)
    }

    /// Merges another VClock into this one, taking the maximum of each entry.
    pub fn merge(&mut self, other: Self) {
        for (actor, other_counter) in other.0 {
            let self_counter = self.0.entry(actor).or_insert(0);
            *self_counter = (*self_counter).max(other_counter);
        }
    }
}
