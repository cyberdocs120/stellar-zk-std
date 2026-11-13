---
title: "infra: production-grade CI/CD, WASM size-guards, and ZkEnv trait architecture"
date: "2026-03-26"
version: "infra"
author: "georgegoldman"
commit: "c86d67116c1f5b60ec2cc98f8a47667d920554dd"
tags: ["ci", "cd", "wasm", "infra", "zkenv"]
---

Establishes the CI/CD foundation for the project:

- GitHub Actions workflow with `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`, and Miri UB checks.
- WASM size guards ensuring guest binaries stay within Soroban limits.
- `ZkEnv` trait architecture separating host-environment concerns from pure cryptographic logic.
- `deny.toml` for license compliance (Apache-2.0 / MIT).

[View commit c86d671](https://github.com/cyberdocs120/stellar-zk-std/commit/c86d67116c1f5b60ec2cc98f8a47667d920554dd)
