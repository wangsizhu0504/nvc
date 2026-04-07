#!/usr/bin/env bash
set -euo pipefail

cargo test --bin nvc remote_node_index::tests::test_list -- --ignored --exact --nocapture
cargo test --test shared_global_prefix exec_uses_shared_prefix_and_global_packages_are_shared -- --ignored --exact --nocapture
