# CmRDTs

[![Tests](https://github.com/johvnik/CmRDTs/actions/workflows/rust.yml/badge.svg)](https://github.com/johvnik/CmRDTs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/cmrdts.svg)](https://crates.io/crates/cmrdts)
[![License](https://img.shields.io/crates/l/cmrdts.svg)](https://github.com/johvnik/CmRDTs/blob/main/LICENSE-MIT)
[![Rust 1.70+](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://rust-lang.org)

A collection of **Commutative Replicated Data Types (CmRDTs)** implemented in pure Rust, suitable for distributed systems and local-first applications.

This library provides a set of simple, serializable, and composable CmRDTs with a focus on network efficiency and practical, real-world use.

## Key Features
- ⚡️ **Efficient Operation-Based Sync:** Sends small, discrete updates instead of entire data states, minimizing network traffic.

- 🌐 **Resilient & Simple Networking:** Works reliably over standard networks with at-least-once, unordered message delivery, removing the need for complex network guarantees.

- 🕰️ **Causal Timestamps:** Uses a Hybrid Logical Clock (HLC) to generate causally-ordered event IDs, ensuring intuitive behavior for LWW types.

- 🔄 **Flexible Sync Strategies:** Supports both lightweight op-based replication and full-state merges for maximum flexibility.

## Core Design

This library's implementation is guided by a pragmatic approach to distributed data synchronization, focusing on providing resilience over real-world networks.

To ensure correctness, each operation is bundled with a minimal amount of metadata, or **causal context** (`AddCtx`).
This context contains a **`VClock`**, which captures the complete history of events the replica has seen.
This history is then used to generate a **`Dot`**, which functions as a Hybrid Logical Clock (HLC) timestamp for the operation.
This single `Dot` provides both **idempotency** (due to its uniqueness) and **causal ordering** (as its internal counter is guaranteed to be greater than any previously seen event).
This robust, unified mechanism is what relaxes the network requirements from complex `exactly-once`, `causal-ordered` delivery to a simpler `at-least-once`, `unordered` model.
The core components that enable this design are:

-  **`Dot` (Unique Operation Identity):** A tuple of `(ActorId, counter)` that serves as a causally-aware, globally unique timestamp. The `counter` is a monotonically increasing logical time generated using HLC logic, while the `ActorId` acts as a tie-breaker.
-  **`VClock` (Causal History):** A map of `ActorId` to the latest `counter` seen from that actor. It captures a replica's knowledge of the system's history and provides the input needed to generate new HLC timestamps.
-  **`AddCtx` (The Causal Context):** The struct containing the `Dot` (the event's timestamp) and the `VClock` (the historical context) that travels with every operation.
-  **`Replica` (The Actor State):** A user-facing wrapper that manages the CRDT state, the actor's local `VClock`. It is responsible for generating new HLC timestamps for each local operation.

While the causal context adds a small overhead to each operation, the payload typically remains significantly smaller than synchronizing the full state of a CvRDT. This design provides the efficiency of an operation-based system with the flexibility to also merge full states, which is useful for an initial sync or for reconciling replicas that have been offline for extended periods.

## Implemented CmRDTs

- **`GCounter`**: A Grow-Only Counter.
- **`PNCounter`**: A Positive-Negative Counter.
- **`LWWRegister`**: A Last-Write-Wins Register.

## Roadmap 𖤓

The near-term goals for this library are:

- [x] Implement `LWWRegister` (Last-Write-Wins Register).
- [ ] Implement `GSet` (Grow-Only Set).
- [ ] Implement `OrSet` (Observed-Remove Set).
- [ ] Implement `RGA` (Replicable Growable Array).

## Testing ⚕

This library is tested using a combination of:

- **Unit tests** for core logic within each module.
- **Property-based tests** with `proptest` to rigorously verify that the CRDTs adhere to their mathematical properties (commutativity, associativity, idempotence) across a wide range of randomized scenarios.

## License

This project is licensed under the MIT License. See the `LICENSE-MIT` file for details.
