# Release Checklist

## Before Tagging

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] fast tests pass
- [ ] real-download smoke tests pass
- [ ] `Cargo.toml` version matches the intended `vX.Y.Z` tag
- [ ] release notes drafted
- [ ] changelog updated
- [ ] README reflects current user-facing behavior

## Release Assets

- [ ] macOS artifact
- [ ] Linux artifact
- [ ] Windows artifact
- [ ] checksums published with artifacts
- [ ] `bash ./scripts/generate-checksums.sh <artifact...>` output saved and attached
- [ ] automated `Release` workflow completed successfully

## Final Review

- [ ] support matrix reviewed
- [ ] known limitations reviewed
- [ ] install instructions verified against release artifacts
