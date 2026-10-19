# bumpus

A verified, fixed single-chunk bump allocator. `src/lib.rs` contains allocation and reclamation proofs; `src/layout.rs` is the trusted `Layout` bridge.

`Bump::{start, end, cursor}` exposes the verified arena bounds and cursor. `try_alloc_layout` specifies the exact aligned address and succeeds exactly when the requested size and alignment fit; failure leaves tracked state unchanged.

`BumpTok::{inst, id, alloc}` identifies an allocation; `alloc()` returns `Alloc { addr, size }`. `BumpTok::agree` relates a live token to `BumpState::addr_of`. Return a subset of permissions and tokens to `reset_to(mark, ...)`; unreturned allocations must start at or above `mark` and remain valid. `reset` returns every allocation and rewinds to `end`; `free` returns every allocation and deallocates the chunk. Neither runs allocation destructors. There is no `Drop`: omitting `free` leaks the chunk.

`smoke` proves pointer reuse after both partial and full reset and reads a value written before the partial rewind.

Checks: `cargo verus verify`, `cargo test --release`, `cargo build --release --all-targets`, and `cargo bench --bench bump`.
