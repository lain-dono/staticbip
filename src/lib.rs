#![no_std]

//! # Static Bip-Buffer
//!
//! A Rust implementation of Simon Cooke's [Bip-Buffer][1]
//!
//! A Bip-Buffer is similar to a circular buffer, but data is inserted in two revolving
//! regions of the buffer space. This allows reads to return contiguous blocks of memory, even
//! if they span a region that would normally include a wrap-around in a circular buffer. It's
//! especially useful for APIs requiring blocks of contiguous memory, eliminating the need to
//! copy data into an interim buffer before use.
//!
//! # Examples
//!
//! ```rust
//! use staticbip::StaticBip;
//!
//! // Creates a 4-element Bip-Buffer of u8
//! let mut buffer = StaticBip::<u8, 4>::default();
//!
//! // Reserves 4 slots for insert
//! buffer.reserve(4).copy_from_slice(&[1, 2, 3, 4]);
//!
//! // Stores the values into an available region,
//! // clearing the existing reservation
//! buffer.commit(3);
//!
//! // Gets the data stored in the region as a contiguous block
//! assert_eq!(buffer.read(), &[1, 2, 3]);
//!
//! // Marks the first two parts of the block as free
//! buffer.decommit(2);
//!
//! // The block should now contain only the last two values
//! assert_eq!(buffer.read(), &[3]);
//! ```
//! [1]: https://www.codeproject.com/articles/3479/the-bip-buffer-the-circular-buffer-with-a-twist

use core::ops::Range;

/// A Bip-Buffer with a fixed capacity.
#[derive(Debug)]
pub struct StaticBip<T, const CAP: usize> {
    /// `A` region
    a: Range<usize>,
    /// `B` region
    b: Range<usize>,
    /// Reserved region
    reserve: Range<usize>,
    /// Backing store
    buffer: [T; CAP],
}

impl<T: Default + Copy, const CAP: usize> Default for StaticBip<T, CAP> {
    #[inline]
    fn default() -> Self {
        Self::new([T::default(); CAP])
    }
}

impl<T, const CAP: usize> StaticBip<T, CAP> {
    /// Creates and allocates a new buffer of `T` elements.
    #[inline]
    pub const fn new(buffer: [T; CAP]) -> Self {
        Self {
            a: 0..0,
            b: 0..0,
            reserve: 0..0,
            buffer,
        }
    }

    /// Size of the backing store.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Number of committed elements.
    ///
    /// This approximates the size of the buffer that will be returned on [`read`](Self::read).
    #[inline]
    pub fn committed(&self) -> usize {
        self.a.end - self.a.start + self.b.end - self.b.start
    }

    /// Number of reserved elements.
    ///
    /// This is the amount of available space for writing data to the buffer.
    #[inline]
    pub fn reserved(&self) -> usize {
        self.reserve.end - self.reserve.start
    }

    /// Whether any space has been reserved or committed in the buffer.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.reserved() == 0 && self.committed() == 0
    }

    /// Clears all regions and reservations.
    ///
    /// Data in the underlying buffer is unchanged.
    #[inline]
    pub fn clear(&mut self) {
        self.a = 0..0;
        self.b = 0..0;
        self.reserve = 0..0;
    }

    /// Returns a mutable buffer containing up to maximum slots for storing data.
    #[inline]
    pub fn reserve_max(&mut self) -> &mut [T] {
        self.reserve(CAP)
    }

    /// Returns a mutable buffer containing up to `count` slots for storing data.
    #[inline]
    pub fn reserve(&mut self, count: usize) -> &mut [T] {
        let space_after_a = self.capacity() - self.a.end;
        let (start, free_space) = if self.b.end > self.b.start {
            (self.b.end, self.a.start - self.b.end)
        } else if space_after_a >= self.a.start {
            (self.a.end, space_after_a)
        } else {
            (0, self.a.start)
        };
        self.reserve = start..start + free_space.min(count);
        &mut self.buffer[self.reserve.clone()]
    }

    /// Commits the data in the reservation, allowing it to be read later.
    ///
    /// If a `len` of `0` is passed in, the reservation will be cleared without making any other changes.
    #[inline]
    pub fn commit(&mut self, len: usize) {
        if len != 0 {
            let to_commit = len.min(self.reserve.end - self.reserve.start);
            if self.a.is_empty() && self.b.is_empty() {
                self.a = self.reserve.start..self.reserve.start + to_commit;
            } else if self.reserve.start == self.a.end {
                self.a.end += to_commit;
            } else {
                self.b.end += to_commit;
            }
        }
        self.reserve = 0..0;
    }

    /// Retrieves available (committed) data as a contiguous block.
    ///
    /// Returns `None` if there is no data available
    #[inline]
    pub fn read(&mut self) -> &mut [T] {
        &mut self.buffer[self.a.clone()]
    }

    /// Marks the first `len` elements of the available data is seen.
    ///
    /// The next time [`read`](Self::read) is called, it will not include these elements.
    #[inline]
    pub fn decommit(&mut self, len: usize) {
        if len >= self.a.end - self.a.start {
            self.a = self.b.clone();
            self.b = 0..0;
        } else {
            self.a.start += len;
        }
    }

    /// Remove the last element in the bip and return it.
    ///
    /// Return a mutable pointer to the removed element,
    /// or `None` if bip doesn't contain commited elements.
    #[inline]
    pub fn pop(&mut self) -> Option<&mut T> {
        self.a
            .next()
            .or_else(|| self.b.next())
            .map(move |index| &mut self.buffer[index])
    }
}
