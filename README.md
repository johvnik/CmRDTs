# CmRDTs

[![Tests](https://github.com/johvnik/CmRDTs/actions/workflows/rust.yml/badge.svg)](https://github.com/johvnik/CmRDTs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/cmrdts.svg)](https://crates.io/crates/cmrdts)
[![License](https://img.shields.io/crates/l/cmrdts.svg)](https://github.com/johvnik/CmRDTs/blob/main/LICENSE-MIT)
[![Rust 1.70+](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://rust-lang.org)

A collection of **Commutative Replicated Data Types (CmRDTs)** implemented in pure Rust, suitable for distributed systems and local-first applications.

This library provides a set of simple, serializable, and composable CmRDTs with a focus on network efficiency and practical, real-world use.

## Key Features
- **Operation-Based (CmRDT):** Instead of syncing the entire data structure (like in state-based CRDTs, or CvRDTs), this library sends only the individual operations (updates). This results in significantly smaller network payloads, increasing efficiency.
- **Resilient Delivery:** Designed for real-world networks. These types work with at-least-once message delivery, removing the need for stricter and more complex exactly-once guarantees from your network infrastructure.
- **Flexible Syncing:** While being operation-based, the CRDTs also expose a merge function. This provides the flexibility to sync full states when needed, offering the best of both op-based and state-based approaches.

## Implemented CmRDTs

- **`GCounter`**: A Grow-Only Counter.
- **`PNCounter`**: A Positive-Negative Counter.

## Roadmap 𖤓

The near-term goals for this library are:

- [ ] Implement `LWWRegister` (Last-Write-Wins Register).
- [ ] Implement `GSet` (Grow-Only Set).
- [ ] Implement `OrSet` (Observed-Remove Set).
- [ ] Implement `RGA` (Replicable Growable Array).

## Testing 𓂀

This library is tested using a combination of:

- **Unit tests** for core logic within each module.
- **Property-based tests** with `proptest` to rigorously verify that the CRDTs adhere to their mathematical properties (commutativity, associativity, idempotence) across a wide range of randomized scenarios.

---
## License

This project is licensed under the MIT License. See the `LICENSE-MIT` file for details.
