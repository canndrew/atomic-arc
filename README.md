## atomic_arc

This Rust crate implements the type `AtomicArc<T>`.

`AtomicArc<T>` is similar to `Arc<T>` except that it is nullable, and it allows
the pointer itself to be set/get atomically.

