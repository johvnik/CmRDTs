use crate::core::{ActorId, AddCtx, CmRDT, Dot, VClock};

/// A Replica manages the state for a single actor, providing the core CRDT functionality.
///
/// The Replica is responsible for wrapping a CRDT data structure (`T`) and handling
/// the generation and application of operations and their causal context.
///
/// ## Hybrid Logical Clock (HLC)
/// This Replica implements a purely logical version of a Hybrid Logical Clock to
/// generate causally-aware timestamps for all operations. This ensures that if
/// event A happens-before event B, then the timestamp of A will be less than B.
///
/// The timestamp is the `Dot` (`{ counter, actor }`):
/// - The `counter` is generated using the logic: `max(local_op_counter, max_vclock_counter) + 1`.
///   This guarantees that any new operation is assigned a timestamp that is strictly
///   greater than any event the Replica has observed, capturing causality.
/// - The `actor` ID serves as a tie-breaker for concurrent events, creating a
///   deterministic total ordering that is essential for LWW (Last-Write-Wins) types.
///
/// This mechanism provides intuitive behavior for all CRDTs without relying on
/// synchronized physical clocks.
#[derive(Debug, Clone)]
pub struct Replica<T: CmRDT> {
    pub actor_id: ActorId,
    op_counter: u64,
    clock: VClock,
    crdt: T,
}

impl<T: CmRDT> Replica<T> {
    /// Creates a new replica for a given actor and an initial CRDT state.
    pub fn new(actor_id: ActorId, crdt: T) -> Self {
        Self {
            actor_id,
            op_counter: 0,
            clock: VClock::default(),
            crdt,
        }
    }

    /// Applies an operation locally and returns the operation and its generated
    /// context, ready to be sent over the network.
    pub fn apply(&mut self, op: T::Op) -> (T::Op, AddCtx) {
        // 1. Find the latest "time" (counter) this replica has seen from any actor.
        let latest_known_time = self.clock.max_counter();

        // 2. Ensure our new op's counter is causally newer than both our last op
        //    and any other op we've seen.
        self.op_counter = latest_known_time.max(self.op_counter) + 1;

        let dot = Dot {
            actor: self.actor_id,
            counter: self.op_counter,
        };

        // Update our own clock with our new operation
        self.clock.0.insert(dot.actor, dot.counter);

        // The context now contains our replica's full, updated clock state
        let ctx = AddCtx {
            dot,
            clock: self.clock.clone(),
        };

        // Apply the op to the local CRDT state
        self.crdt.apply(op.clone(), ctx.clone());

        // Return the op and context to the caller
        (op, ctx)
    }

    /// Applies a remote operation and merges its causal context.
    pub fn apply_remote(&mut self, op: T::Op, ctx: AddCtx) {
        // 1. Apply the operation to the underlying CRDT.
        self.crdt.apply(op, ctx.clone());

        // 2. Merge the incoming clock to update our own causal knowledge.
        self.clock.merge(ctx.clock);
    }

    pub fn read(&self) -> T::Value {
        self.crdt.read()
    }

    pub fn merge(&mut self, remote_crdt: T, remote_clock: VClock) {
        self.crdt.merge(remote_crdt);
        self.clock.merge(remote_clock);
    }

    pub fn state(&self) -> &T {
        &self.crdt
    }

    pub fn clock(&self) -> &VClock {
        &self.clock
    }
}
