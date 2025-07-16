# CmRDTs

A collection of Commutative Replicated Data Types (CmRDTs) implemented in pure Rust.

This library provides a set of simple, serializable, and composable CRDTs suitable for distributed systems and local-first applications.

---
## Implemented CRDTs

- **`GCounter`**: A Grow-Only Counter.
- **`PNCounter`**: A Positive-Negative Counter.

---
## Roadmap 🗺️

The near-term goals for this library are:

- [ ] Add comprehensive testing and documentation for all data types.
- [ ] Implement `LWWRegister` (Last-Write-Wins Register).
- [ ] Implement `GSet` (Grow-Only Set).
- [ ] Implement `OrSet` (Observed-Remove Set).
- [ ] Implement `RGA` (Replicable Growable Array).

---
## License

This project is licensed under the MIT License. See the `LICENSE-MIT` file for details.
