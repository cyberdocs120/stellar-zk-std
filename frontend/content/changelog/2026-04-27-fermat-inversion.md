---
title: "feat: constant-time Fermat-based modular inversion (Issue #8)"
date: "2026-04-27"
version: "minor"
author: "Miracle656"
commit: "dbc26d5f15155263879b68144ba700a11a94f6f9"
closes: ["#8"]
tags: ["crypto", "constant-time", "inversion", "soroban-zk-core"]
---

Implements Issue #8. Constant-time Fermat-based modular inversion in `soroban-zk-core`,
safe for use in WASM environments without variable-time branches.

[View commit dbc26d5](https://github.com/cyberdocs120/stellar-zk-std/commit/dbc26d5f15155263879b68144ba700a11a94f6f9)
