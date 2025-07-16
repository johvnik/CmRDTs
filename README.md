# CmRDTs

[![Tests](https://github.com/johvnik/CmRDTs/actions/workflows/rust.yml/badge.svg)](https://github.com/johvnik/CmRDTs/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/cmrdts.svg)](https://crates.io/crates/cmrdts)
[![License](https://img.shields.io/crates/l/cmrdts.svg)](https://github.com/johvnik/CmRDTs/blob/main/LICENSE-MIT)
[![Rust 1.70+](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://rust-lang.org)

A collection of Commutative Replicated Data Types (CmRDTs) implemented in pure Rust.

This library provides a set of simple, serializable, and composable CRDTs suitable for distributed systems and local-first applications.

---
## Implemented CRDTs

- **`GCounter`**: A Grow-Only Counter.
- **`PNCounter`**: A Positive-Negative Counter.

---
## Testing 🧪

This library is tested using a combination of:

- **Unit tests** for core logic within each module.
- **Property-based tests** with `proptest` to rigorously verify that the CRDTs adhere to their mathematical properties (commutativity, associativity, idempotence) across a wide range of randomized scenarios.

---
## Roadmap 🗺️

The near-term goals for this library are:

- [ ] Implement `LWWRegister` (Last-Write-Wins Register).
- [ ] Implement `GSet` (Grow-Only Set).
- [ ] Implement `OrSet` (Observed-Remove Set).
- [ ] Implement `RGA` (Replicable Growable Array).

---
## License

This project is licensed under the MIT License. See the `LICENSE-MIT` file for details.
