# Testing Strategy

This repository keeps real Node downloads in test coverage, but it does not run the full network-heavy matrix on every pull request.

## Tiers

- `fast`
  - Offline and quick
  - No npm registry access
  - Runs on every pull request
- `real-download-smoke`
  - Downloads real Node distributions from the official mirror
  - Verifies the main install/exec/shared-global-prefix path
  - Runs on every pull request
- `full-real-download`
  - Runs the heavier real-download cases across platforms
  - Intended for scheduled runs and pre-release confidence

## Commands

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --bin nvc remote_node_index::tests::test_list -- --ignored --exact --nocapture
cargo test --test shared_global_prefix exec_uses_shared_prefix_and_global_packages_are_shared -- --ignored --exact --nocapture
```

## Rules

- Default PR checks must not depend on the npm registry.
- Real Node downloads are required coverage and stay in CI.
- Heavier cross-platform download checks move to scheduled or manually triggered workflows so they do not dominate every PR cycle.
