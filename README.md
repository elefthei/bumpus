# bumpus

A bump allocator with a machine-checked proof. It carves memory out of one fixed chunk by moving a cursor downward, and the [Verus](https://github.com/verus-lang/verus) proofs in `src/` pin down when an allocation succeeds, where it lands, and what a reset hands back. The chunk never grows. When it's full, you get an error.

```toml
[dependencies]
bumpus = "0.1"
```

Works in `no_std` with `alloc`, on 32-bit and 64-bit targets.

## Using it

Most code wants one of the two containers. `TypedArena<T>` stores `Copy` values, `SliceArena<T>` stores copied slices, and both give you an index back on push.

```rust
use bumpus::{AllocErr, TypedArena};

let mut arena = TypedArena::with_capacity(64).unwrap(); // room for 8 u64s
for i in 0..8u64 {
    arena.try_push(i).unwrap();
}
assert_eq!(arena.try_push(99), Err(AllocErr::Exhausted));
assert_eq!(arena.get(3), Some(&3));
arena.free();
```

Call `free` yourself. There is no `Drop` impl, so an arena that falls out of scope leaks its chunk.

Underneath sits `Bump`, the raw allocator. It's built for Verus callers: each allocation returns a pointer together with a ghost permission and token, and you hand those back to reset or free the arena.

## What the proofs guarantee

These hold for verified callers that keep the arena's tracked state:

- An allocation succeeds if and only if the request, after alignment, fits below the cursor. You get an aligned pointer inside the chunk that overlaps no other live allocation. A failed allocation changes nothing.
- `reset_to(mark)` rewinds the cursor, provided every allocation you still hold sits at or above `mark`. The ones you keep stay valid and keep their contents. `reset` takes everything back; `free` returns the chunk to the global allocator.
- In the containers, pushing a `w`-byte value or slice uses exactly `w` bytes. Same-type pushes never pad, so `n * size_of::<T>()` bytes hold `n` values. A failed push leaves the contents alone.

Errors come in three kinds: `Capacity` (asked for zero bytes or too many), `Global` (the system allocator refused), and `Exhausted` (doesn't fit in what's left).

## What they don't

Leaks are out of scope. Verus won't complain about an arena dropped without `free`, and a panic halfway through a container push can lose the chunk. Unwinding isn't verified either. No destructors ever run, which is why the containers require `T: Copy`.

The trusted base is small: the call into the global allocator, the function that turns a slice permission into a `&[T]`, two `Layout` specs in `src/layout.rs`, and vstd's own axioms. Proofs were checked on x86_64.

## Checking it

```sh
cargo verus verify
cargo test --release
cargo bench --bench bump
```

## Speed

The benchmark times 10,000 allocations plus a reset, against `bumpalo` 3.20.3. `small` allocates `u8`, `big` allocates `[usize; 32]`; `const` lets the optimizer see the layout and `opaque` hides it. Numbers are bumpus time divided by bumpalo time, so lower favors bumpus. One release run on x86_64 Linux (WSL2):

| Case | bumpus / bumpalo |
| --- | ---: |
| small/const | 0.62× |
| small/opaque | 0.47× |
| big/const | 0.39× |
| big/opaque | 0.43× |

That's one run on one machine. Read it as "not slower than bumpalo," not as a promised 2× speedup.

## License

MIT
