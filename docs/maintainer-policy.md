# Maintainer Policy

## Product Boundary

`nvc` is maintained as an independent project.

It may reference upstream `fnm` behavior, but it is not a direct mirror and does not merge upstream changes blindly.

## Upstream Sync Rules

Before adopting upstream behavior:

1. Confirm the change is compatible with `nvc` product direction.
2. Review licensing implications.
3. Re-implement or adapt intentionally instead of bulk-merging by default.
4. Validate the result with real-download smoke coverage and release checks.

## Licensing Boundary

- `nvc` currently remains MIT licensed.
- Any future upstream-inspired work must be reviewed against the project licensing strategy before adoption.
- When behavior is aligned with upstream, document that alignment explicitly rather than implying direct source import.

## Quality Gate

A change is not ready unless it passes:

- formatting and lint checks
- fast tests
- real-download smoke validation where relevant
- documentation updates for user-visible behavior

## Release Discipline

Each release should include:

- changelog update
- artifact validation
- support matrix review
- known limitations review
