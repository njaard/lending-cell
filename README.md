LendingCell is a mutable container that
allows you to get an owned reference to the same
object. When the owned reference is dropped,
ownership returns to the original container.

As opposed to a [`std::cell::Cell`] which moves Rust's
_ownership_ rules to runtime, a `LendingCell` moves Rust's
_lifetime_ rules to runtime.

The value of a `LendingCell` is present at
construction-time, but you can convert it to a
`BorrowedCell<T>` by calling [`LendingCell::to_borrowed`].
While that `BorrowedCell` lives (that is, until it is dropped),
calling [`LendingCell::try_get`] will return `None`. The
`BorrowedCell` has exclusive access to your type, as though you
have a directly owned instance of the `T`, and so it is `Send`
as long as `T` is Send. At last, when you drop the `BorrowedCell`,
the `T` is returned to the `LendingCell`.

If you drop your `LendingCell` before dropping the `BorrowedCell`,
then the `T` is dropped at the time you drop the `BorrowedCell`.

The invariants that Rust's memory safety rules enforce, like
the single-mutable reference rule, are therefor partially ensured
at compile time (you can't get two mutable references to the
`T`), but also partially at runtime (while the `BorrowedCell`
is active, the `LendingCell` behaves as though it is an `Option`
containing `None`.

