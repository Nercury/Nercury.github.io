# Dev brief

Working conventions for the generator (`crates/blog`) and diff tool (`crates/sitediff`), plus a log
of noteworthy uses of new Rust features.

## Toolchain

- Rust **nightly**, edition **2024** (latest at time of writing; bump when a newer edition lands).
- **No feature gates.** Nightly is used only to get features that are already *stabilized* but have
  not yet reached beta/stable. `#![feature(...)]` is not allowed; this is enforced by
  `unstable_features = "forbid"` in `[workspace.lints.rust]` (verified: adding a gate fails the build).
- Consequence: the code is at most ~12 weeks ahead of stable. Any breakage fallback is simply
  "this will compile on stable by release X".
- `rust-toolchain.toml` tracks floating `nightly`. Baseline when the project started:
  `rustc 1.100.0-nightly (330d31712 2026-09-17)`.
- Before the first CI deploy (MIGRATION Phase 7), pin a dated nightly (`nightly-YYYY-MM-DD`) so CI
  and local builds agree; once everything used has reached stable, switching the pin to `stable`
  is an option.
- Caveat: the lint cannot distinguish "stabilized on nightly" from "stable for years"; it only
  blocks gates. Whether an API is newly stabilized is visible in its docs (`Stable since 1.xx.0`).
  Dependencies are not covered by the lint — avoid crates that require nightly gates.

## Feature log

Newly stabilized features (not yet on stable when adopted) that turned out to be interesting.
Record: feature, stabilized-in version, where it is used, what it bought us.

| Feature | Stable in | Used in | Why it was interesting |
|---------|-----------|---------|------------------------|
| _none yet_ | | | |

## Notes on interesting uses

_(Longer write-ups go here when a use of a new feature is worth explaining.)_
