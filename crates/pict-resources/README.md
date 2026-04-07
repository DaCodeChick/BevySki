# pict-resources

PICT resource decoding and PNG conversion used by BevySki.

## Attribution

This crate includes code adapted from `pict` (pict-rs) by cyco:
- Repository: https://codeberg.org/cyco/pict-rs
- Crate: https://crates.io/crates/pict
- License: Apache-2.0

The following modules are derived from that implementation and then modified for this project:
- `src/drawing_context.rs`
- `src/shared.rs`
- `src/v1.rs`
- `src/v2.rs`

BevySki-specific integration and extraction glue in `src/lib.rs` is maintained in this repository.
