# Held-back major dependency bumps

Written after the 2026-07-17 dependency sweep. These are **not** currently broken on `main` —
they're major-version bumps that `cargo upgrade` (compatible-only) correctly skipped. Renovate's
open PR (`renovate/non-major-dependencies` #4, mislabeled — both of these are 0.x version jumps
that Renovate's default classifier treats as "non-major" even though Cargo's own semver rules
treat them as breaking) fails CI. Do not merge that PR as-is.

## ctor 0.6.3 → 0.13.0 *(confirmed break)*

**Fails with:**
```
error: Missing unsafe keyword in #[ctor] annotation. Use #[ctor(unsafe)].
  --> src/test_utils.rs:21:1
   |
21 | #[ctor::ctor]
   | ^^^^^^^^^^^^^
```

`ctor` 0.13 (part of a project rebrand to "linktime") requires an explicit `unsafe` acknowledgment
on the attribute, since ctor-registered functions run before `main` in a context where Rust's
usual safety guarantees (e.g. no other statics initialized yet) don't fully hold — the new
version makes callers say so explicitly.

**Migration**: one-line fix in `src/test_utils.rs`:
```rust
#[ctor::ctor]
```
becomes
```rust
#[ctor::ctor(unsafe)]
```
Then bump the `Cargo.toml` requirement to `ctor = "0.13"` (or `"1"` if you go all the way to
1.0.9, the actual latest — check whether 0.13 → 1.0 changes anything else first). This looks
mechanical, but do read the [ctor release notes](https://github.com/mmastrac/rust-ctor/releases)
for *why* the `unsafe` marker exists before rubber-stamping it — worth a moment's thought about
whether `init_test_logger`'s ctor-run init is actually sound in that pre-main context (it almost
certainly is, this is a common and safe pattern for test-logger setup, but confirm rather than
assume).

## saphyr 0.0.6 → 0.0.11

Used for YAML parsing (`saphyr::Yaml` in `src/config/parser.rs`). `0.0.x` versions have no
semver compatibility guarantee at all (every patch is technically a breaking-change slot under
Cargo's own caret rules, which is exactly why `cargo upgrade` correctly refused to touch this
automatically). Not diagnosed beyond that — no CI run has attempted this yet, since ctor's
failure blocks compilation before saphyr's actual usage is exercised. Check `saphyr`'s changelog
between 0.0.6 and 0.0.11 for any YAML-parsing API changes before bumping; given this crate is
pre-alpha by its own versioning, expect the API to have moved.

## Suggested order

1. **ctor** first — it's the one-line fix above, low risk, and unblocks CI from even reaching the
   saphyr code path.
2. **saphyr** second, once ctor is sorted — needs its own careful look at the YAML parsing code
   in `src/config/parser.rs` and `src/test_utils.rs`'s YAML builders.

## Until then

Renovate will keep proposing this bundle — it doesn't know these break, only that they're
semver-compatible per its own (looser, 0.x-tolerant) classification. Either close each new PR as
it appears, or add a `packageRules` entry to `renovate.json` disabling major-version PRs for this
repo (matching the pattern already used in `the-bannered-mare`'s `renovate.json`) if you'd rather
stop seeing them — though note that rule alone might not catch these specific 0.x bumps if
Renovate's classifier doesn't consider them "major" in the first place; may need
`matchCurrentVersion` / `matchPackageNames` rules targeting `ctor` and `saphyr` specifically.
