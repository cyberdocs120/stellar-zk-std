---
title: "feat(crypto): complete BN254 arithmetic and ElGamal bridge"
date: "2026-04-29"
version: "minor"
author: "georgegoldman"
commit: "a63a47d6b40da1ae9b14ed53ee26dbee7e813372"
closes: ["#9", "#12", "#17", "#20", "#21"]
tags: ["crypto", "bn254", "elgamal", "soroban-zk-core"]
---

Closes #9, #12, #17, #20, #21. Major arithmetic milestone for `soroban-zk-core`:

- BN254 base field (`Fq`) and scalar field (`Fr`) arithmetic.
- `Fr` subtraction with modular underflow protection.
- G1Projective Jacobian point addition and doubling.
- Constant-time `g1_scalar_mul` using bitwise `ct_select` — no scalar leakage via timing side-channels.
- Jacobian → Affine coordinate conversion (`to_affine`).
- `ElGamalCiphertext` API synchronised with `shielded-asset-template`.
- 100% passing tests and successful WASM target compilation.

[View commit a63a47d](https://github.com/cyberdocs120/stellar-zk-std/commit/a63a47d6b40da1ae9b14ed53ee26dbee7e813372)
