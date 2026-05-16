# Contributing to bereme-protocol

Wire-protocol crates have a special contribution surface: changing a type touches *every* device in the field and every cloud instance that speaks to them. Please read this before opening a PR.

## Anatomy of a change

| Kind of change | What it does to the wire | What to do |
|---|---|---|
| **Add a new optional field** | Older peers ignore it (serde default = None / Vec::new). | OK as a patch bump (0.1.x → 0.1.x+1) if the field is `Option<T>` or `Vec<T>` (`#[serde(default)]`). |
| **Add a new enum variant** | Older peers reject it (the dispatcher logs + drops the frame). | Minor bump (0.x.y → 0.x+1.0). Document in the corresponding enum's doc-comment. |
| **Rename a field or change a serde tag** | Older peers will fail to deserialize. | Major bump (0.x.y → 1.0.0). Coordinate the cloud + device release window so both sides ship together. |
| **Remove a variant** | Same as above — wire break. | Major bump. |

If you're not sure which category your change falls into, open an issue first.

## Local development

```sh
cargo build
cargo test
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
```

All five must pass before pushing.

`tests/round_trip.rs` and `tests/adversarial.rs` together cover every public type. Add a test to `round_trip.rs` for any new variant (proves it serializes), and an entry to `adversarial.rs` for any new constant or wire-shape pin (proves it's locked).

## Release procedure

1. Update `Cargo.toml`'s `version`.
2. `cargo test` + `cargo fmt --check` + `cargo clippy --all-targets --all-features -- -D warnings` clean.
3. `git tag -a bereme-protocol-vX.Y.Z -m "release notes"` on `main`.
4. Push the tag: `git push origin bereme-protocol-vX.Y.Z`.
5. Open coordinated PRs in [`kvm-rust`](https://github.com/Waiel5/kvm-rust) and [`cloud-rust`](https://github.com/Waiel5/cloud-rust) bumping the `bereme-protocol` git-dep tag.

Both consumer PRs must land within the same merge window so device and cloud stay in lock-step on the wire.

## Coding conventions

- `#![forbid(unsafe_code)]` on the crate root — no exceptions.
- Every public struct field that can be `None` on the wire is `Option<T>` with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
- Tag values are `#[serde(rename_all = "snake_case")]`. Don't add manual `rename = "..."` unless you absolutely must — and document why if you do.
- Doc-comments on every public type and field. The crate's reader is the *implementor on the other side of the wire*; they need to know what each field means without reading the source.

## Reporting a security issue

See [`SECURITY.md`](./SECURITY.md). Wire-shape attacks (e.g. crafted JSON that crashes the dispatcher) qualify.
