---
title: "feat: field element serialization to/from canonical bytes (Issue #18)"
date: "2026-04-27"
version: "minor"
author: "Miracle656"
commit: "ffbd41aa573c1eb1f004fb685f213bf0b5da0033"
closes: ["#18"]
tags: ["serialization", "bn254", "fr", "fq", "soroban-zk-core"]
---

Implements Issue #18.

- `FQ_MODULUS` constant (BN254 base field prime p) added to `Bn254`.
- `fr_to_bytes` / `fr_from_bytes` with strict range check against r.
- `fq_to_bytes` / `fq_from_bytes` with strict range check against p.
- 13 tests: round-trip identity, boundary values, rejection of inputs ≥ modulus.
- No heap allocation — fixed `[u8; 32]` arrays, snarkjs/circom big-endian compatible.

[View commit ffbd41a](https://github.com/cyberdocs120/stellar-zk-std/commit/ffbd41aa573c1eb1f004fb685f213bf0b5da0033)
