# Security Policy

`bereme-protocol` is the wire-type definition crate for the Bereme device ↔ cloud WebSocket. The crate itself has no I/O and no `unsafe`, but a bug in *the types* propagates to every cloud instance and every device firmware that depends on it.

## Reporting a vulnerability

**Do not file public GitHub issues for security bugs.**

Private channels:

1. **GitHub private vulnerability reporting** (preferred):
   <https://github.com/Waiel5/bereme-protocol/security/advisories/new>
2. **Email:** `security@bereme.com`.

What we'd like in the report:

- The type or constant the issue is in.
- A proof-of-concept input (JSON the dispatcher would receive) that triggers the bug.
- Your assessment of impact: does it crash the deserializer? Force unbounded allocation? Allow type confusion? Bypass a wire-shape invariant?

## What counts as in-scope

- **Deserializer panic** on input the dispatcher legitimately receives (any JSON that came in on a WS Text frame).
- **Unbounded allocation or CPU** triggered by a payload below `MAX_FRAME_BYTES`.
- **Type confusion** where serde accepts a frame that violates a documented invariant (e.g. a `RegisterMessage` deserializes when the `type` tag is absent).
- **Forward-compat regression** where a future cloud variant the device doesn't know about causes anything worse than a single warn-log + frame drop.

## What's out of scope

- Bugs in `serde_json` itself (please file with [serde-rs/json](https://github.com/serde-rs/json) — we subscribe via RUSTSEC).
- Issues that require an attacker with admin access on the cloud (e.g. "an admin can send a malicious cmd payload"). The crate's threat model assumes the cloud is trusted.

## Disclosure timeline

We will acknowledge within 72 hours and aim to ship a patch tag (`bereme-protocol-vX.Y.Z+1`) within 14 days for critical defects, 30 days for everything else.

A fix lands as a coordinated three-step release:

1. Patch `bereme-protocol`, push tag.
2. Patch both `kvm-rust` and `cloud-rust` to bump their git-dep tag in the same merge window.
3. Public disclosure + advisory once the consumer releases have rolled out.
