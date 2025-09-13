# CmRDTs

[![Tests](https://github.com/johvnik/CmRDTs/actions/workflows/rust.yml/badge.svg)](https://github.com/johvnik/CmRDTs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/cmrdts.svg)](https://crates.io/crates/cmrdts)
[![License](https://img.shields.io/crates/l/cmrdts.svg)](https://github.com/johvnik/CmRDTs/blob/main/LICENSE-MIT)
[![Rust 1.70+](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://rust-lang.org)

A collection of **Commutative Replicated Data Types (CmRDTs)** implemented in pure Rust, suitable for distributed systems and local-first applications.

This library provides a set of simple, serializable, and composable CmRDTs with a focus on network efficiency and practical, real-world use.

## Key Features
- ⚡️ Efficient Operation-Based Sync: Sends small, discrete updates instead of entire data states, minimizing network traffic.

- 🌐 Resilient & Simple Networking: Works reliably over standard networks with at-least-once, unordered message delivery, removing the need for complex network guarantees.

- 🔄 Flexible Sync Strategies: Supports both lightweight op-based replication and full-state merges for maximum flexibility.

## Core Design

This library's implementation is guided by a pragmatic approach to distributed data synchronization, focusing on providing resilience over real-world networks.

To ensure correctness, each operation is bundled with a minimal amount of metadata, or **causal context** (`AddCtx`). This context makes replication robust by providing two key guarantees. First, it contains a **`Dot`** that makes every operation idempotent, relaxing the network requirement from a complex *exactly-once* to a simpler ***at-least-once*** delivery. Second, it includes a **`VClock`** to track causal history, allowing operations to be correctly integrated even when they arrive out of order. This relaxes the delivery requirement from *causal-ordered* to simple ***unordered*** delivery.

The core components that enable this design are:

-  **`Dot` (Unique Operation Identity):** A tuple of `(ActorId, counter)` that gives every operation a globally unique and immutable identifier.
-  **`VClock` (Causal History):** A map of `ActorId` to the latest `counter` seen from that actor, capturing a replica's knowledge of the system's history.
-  **`AddCtx` (The Causal Context):** The struct containing the `Dot` and `VClock` that travels with every operation, providing all necessary metadata for a safe update.
-  **`Replica` (The Actor State):** A user-facing wrapper that manages the CRDT state, the actor's local `VClock`, and is responsible for generating and applying the `AddCtx`.

While the causal context adds a small overhead to each operation, the payload typically remains significantly smaller than synchronizing the full state of a CvRDT. This design provides the efficiency of an operation-based system with the flexibility to also merge full states, which is useful for an initial sync or for reconciling replicas that have been offline for extended periods.

## Implemented CmRDTs

- **`GCounter`**: A Grow-Only Counter.
- **`PNCounter`**: A Positive-Negative Counter.

## Roadmap 𖤓

The near-term goals for this library are:

[ ] Implement `LWWRegister` (Last-Write-Wins Register).
[ ] Implement `GSet` (Grow-Only Set).
[ ] Implement `OrSet` (Observed-Remove Set).
[ ] Implement `RGA` (Replicable Growable Array).

## Testing ⚕

This library is tested using a combination of:

- **Unit tests** for core logic within each module.
- **Property-based tests** with `proptest` to rigorously verify that the CRDTs adhere to their mathematical properties (commutativity, associativity, idempotence) across a wide range of randomized scenarios.

## License

This project is licensed under the MIT License. See the `LICENSE-MIT` file for details.
