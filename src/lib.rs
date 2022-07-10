//! LendingCell is a mutable container that
//! allows you to get an owned reference to the same
//! object. When the owned reference is dropped,
//! ownership returns to the original container.
//!
//! As opposed to a [`std::cell::Cell`] which moves Rust's
//! _ownership_ rules to runtime, a `LendingCell` moves Rust's
//! _lifetime_ rules to runtime.
//!
//! The value of a `LendingCell` is present at
//! construction-time, but you can convert it to a
//! `BorrowedCell<T>` by calling [`LendingCell::to_borrowed`].
//! While that `BorrowedCell` lives (that is, until it is dropped),
//! calling [`LendingCell::try_get`] will return `None`. The
//! `BorrowedCell` has exclusive access to your type, as though you
//! have a directly owned instance of the `T`, and so it is `Send`
//! as long as `T` is Send. At last, when you drop the `BorrowedCell`,
//! the `T` is returned to the `LendingCell`.
//!
//! If you drop your `LendingCell` before dropping the `BorrowedCell`,
//! then the `T` is dropped at the time you drop the `BorrowedCell`.
//!
//! The invariants that Rust's memory safety rules enforce, like
//! the single-mutable reference rule, are therefor partially ensured
//! at compile time (you can't get two mutable references to the
//! `T`), but also partially at runtime (while the `BorrowedCell`
//! is active, the `LendingCell` behaves as though it is an `Option`
//! containing `None`.

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// A container that allows borrowing without lifetimes.
///
/// ```rust
/// # use lending_cell::*;
/// let mut lender = LendingCell::new("borrowed");
/// let borrowed = lender.to_borrowed();
/// assert!(lender.try_get().is_none()); // `lender` is empty because it was borrowed
/// assert_eq!(*borrowed, "borrowed"); // it's certain that `borrowed` is valid while in scope
/// drop(borrowed); // dropping `borrowed` returns its value to `lender`
/// assert!(lender.try_get().is_some());
/// ```
pub struct LendingCell<T> {
    thing: Arc<UnsafeCell<T>>,
}

// SAFETY: type imitates T ownership
unsafe impl<T: Sync> Sync for LendingCell<T> {}
unsafe impl<T: Send> Send for LendingCell<T> {}

impl<T> LendingCell<T> {
    /// Creates a new LendingCell with the given value
    pub fn new(thing: T) -> Self {
        Self {
            thing: Arc::new(UnsafeCell::new(thing)),
        }
    }

    /// Get a reference to the contained value if it wasn't borrowed with
    /// [`LendingCell::to_borrowed`]
    pub fn try_get(&self) -> Option<&T> {
        if Arc::strong_count(&self.thing) == 1 {
            Some(unsafe { &*self.thing.get() })
        } else {
            None
        }
    }

    /// Get a reference to the contained value if it wasn't borrowed with
    /// [`LendingCell::to_borrowed`], otherwise panic
    pub fn get(&self) -> &T {
        self.try_get().unwrap()
    }

    /// Get a mutable reference the contained value if it wasn't borrowed with
    /// [`LendingCell::to_borrowed`]
    pub fn try_get_mut(&mut self) -> Option<&mut T> {
        Arc::get_mut(&mut self.thing).map(|c| c.get_mut())
    }

    /// Get a mutable reference the contained value if it wasn't borrowed with
    /// [`LendingCell::to_borrowed`], otherwise panic
    pub fn get_mut(&mut self) -> &mut T {
        self.try_get_mut().unwrap()
    }

    /// Take the contained value and returned it in an owned object if it
    /// isn't already borrowed, otherwise panic.
    pub fn to_borrowed(&mut self) -> BorrowedCell<T> {
        self.try_to_borrowed().unwrap()
    }

    /// Take the contained value and returned it in an owned object if it
    /// isn't already borrowed.
    pub fn try_to_borrowed(&mut self) -> Option<BorrowedCell<T>> {
        if Arc::strong_count(&self.thing) == 1 {
            Some(BorrowedCell {
                thing: Arc::clone(&self.thing),
            })
        } else {
            None
        }
    }

    /// Destroy the container and return the contained object if it isn't
    /// being borrowed already. If it fails, return myself `LendingCell`
    pub fn try_into_inner(self) -> Result<T, Self> {
        Arc::try_unwrap(self.thing)
            .map(|x| x.into_inner())
            .map_err(|a| LendingCell { thing: a })
    }
}

/// The container that ensures you have borrowed the [`LendingCell`].
pub struct BorrowedCell<T> {
    thing: Arc<UnsafeCell<T>>,
}

// SAFETY: type imitates either a mutable reference or an ownership
unsafe impl<T: Send> Send for BorrowedCell<T> {}
unsafe impl<T: Sync> Sync for BorrowedCell<T> {}

impl<T> Deref for BorrowedCell<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.thing.get() }
    }
}

impl<T> DerefMut for BorrowedCell<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.thing.get() }
    }
}
