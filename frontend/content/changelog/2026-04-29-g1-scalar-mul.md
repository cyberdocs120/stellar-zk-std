---
title: "feat(crypto): constant-time g1_scalar_mul (double-and-add)"
date: "2026-04-29"
version: "minor"
author: "georgegoldman"
commit: "1a0ddcd20600e1a6538749b458cf67a9b4f6003d"
closes: ["#12"]
tags: ["crypto", "bn254", "constant-time", "soroban-zk-core"]
---

Closes #12. Implements `g1_scalar_mul` over BN254 G1:

- Constant-time double-and-add scanning all 254 scalar bits.
- `ct_select` eliminates branching on scalar bits.
- Edge-case handling for scalar = 0 (identity) and scalar = 1.
- Test suite: group order identity, multiplicative identity, doubling consistency.

[View commit 1a0ddcd](https://github.com/cyberdocs120/stellar-zk-std/commit/1a0ddcd20600e1a6538749b458cf67a9b4f6003d)
