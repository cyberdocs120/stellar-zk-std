---
title: "feat: Poseidon2 sponge construction for BN254 (CAP-0075, Issue #2)"
date: "2026-04-27"
version: "minor"
author: "Salmatcre8"
commit: "65feaed6e0bf7646c619973bc7062a190b9ac8ba"
closes: ["#2"]
tags: ["poseidon2", "hash", "cap-0075", "soroban-zk-std"]
---

Implements Issue #2. Poseidon2 sponge over BN254 Fr (t=3, rate=2, capacity=1):

- Calls `env.crypto_hazmat().poseidon2_permutation()` — one host call per permutation, zero guest-side loop overhead.
- `hash_to_field()` compatible with Noir/Circom Poseidon2 constraints.
- KAT verified against `soroban-env-host-25.0.1`.

[View commit 65feaed](https://github.com/cyberdocs120/stellar-zk-std/commit/65feaed6e0bf7646c619973bc7062a190b9ac8ba)
