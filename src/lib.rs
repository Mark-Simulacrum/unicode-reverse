#![no_std]

//! The [`reverse_grapheme_clusters_in_place`][0] function reverses a string slice in-place without
//! allocating any memory on the heap.  It correctly handles multi-byte UTF-8 sequences and
//! grapheme clusters, including combining marks and astral characters such as Emoji.
//!
//! ## Example
//!
//! ```rust
//! use unicode_reverse::reverse_grapheme_clusters_in_place;
//!
//! let mut x = "man\u{0303}ana".to_string();
//! println!("{}", x); // prints "mañana"
//!
//! reverse_grapheme_clusters_in_place(&mut x);
//! println!("{}", x); // prints "anañam"
//! ```
//!
//! ## Background
//!
//! As described in [this article by Mathias Bynens][1], naively reversing a Unicode string can go
//! wrong in several ways. For example, merely reversing the `chars` (Unicode Scalar Values) in a
//! string can cause combining marks to become attached to the wrong characters:
//!
//! ```rust
//! let x = "man\u{0303}ana";
//! println!("{}", x); // prints "mañana"
//!
//! let y: String = x.chars().rev().collect();
//! println!("{}", y); // prints "anãnam": Oops! The '~' is now applied to the 'a'.
//! ```
//!
//! Reversing the [grapheme clusters][2] of the string fixes this problem:
//!
//! ```rust
//! extern crate unicode_segmentation;
//! use unicode_segmentation::UnicodeSegmentation;
//!
//! # fn main() {
//! let x = "man\u{0303}ana";
//! let y: String = x.graphemes(true).rev().collect();
//! println!("{}", y); // prints "anañam"
//! # }
//! ```
//!
//! The `reverse_grapheme_clusters_in_place` function from this crate performs this same operation,
//! but performs the reversal in-place rather than allocating a new string.
//!
//! ## Algorithm
//!
//! The implementation is very simple. It makes two passes over the string's contents:
//!
//! 1. For each grapheme cluster, reverse the bytes within the grapheme cluster in-place.
//! 2. Reverse the bytes of the entire string in-place.
//!
//! After the second pass, each grapheme cluster has been reversed twice, so its bytes are now back
//! in their original order, but the clusters are now in the opposite order within the string.
//!
//! ## no_std
//!
//! This crate does not depend on libstd, so it can be used in [`no_std` projects][3].
//!
//! [0]: fn.reverse_grapheme_clusters_in_place.html
//! [1]: https://mathiasbynens.be/notes/javascript-unicode
//! [2]: http://www.unicode.org/reports/tr29/#Grapheme_Cluster_Boundaries
//! [3]: https://doc.rust-lang.org/book/no-stdlib.html

extern crate unicode_segmentation;

use core::slice;
use core::str;
use unicode_segmentation::UnicodeSegmentation;

/// Reverse a Unicode string in-place without allocating.
///
/// This function reverses a string slice in-place without allocating any memory on the heap.  It
/// correctly handles multi-byte UTF-8 sequences and grapheme clusters, including combining marks
/// and astral characters such as Emoji.
///
/// See the [crate-level documentation](index.html) for more details.
///
/// ## Example
///
/// ```rust
/// extern crate unicode_reverse;
/// use unicode_reverse::reverse_grapheme_clusters_in_place;
///
/// fn main() {
///     let mut x = "man\u{0303}ana".to_string();
///     println!("{}", x); // prints "mañana"
///
///     reverse_grapheme_clusters_in_place(&mut x);
///     println!("{}", x); // prints "anañam"
/// }
/// ```
pub fn reverse_grapheme_clusters_in_place(s: &mut str) {
    // Part 1: Reverse the bytes within each grapheme cluster.
    // This does not preserve UTF-8 validity. We must guarantee this `reverse` is
    // undone before the data is accessed as `str` again.
    {
        let mut tail = &mut s[..];
        loop {
            // Advance to the next grapheme cluster:
            let len = match tail.graphemes(true).next() {
                Some(grapheme) => grapheme.len(),
                None => break
            };
            let (head, new_tail) = {tail}.split_at_mut(len);
            tail = new_tail;

            // Reverse the bytes within this grapheme cluster.
            let bytes = unsafe {
                let head = head;
                // This is safe because `head` is &mut str so guaranteed not to be aliased.
                slice::from_raw_parts_mut(head.as_ptr() as *mut u8, head.len())
            };
            bytes.reverse();
        }
    }

    // Part 2: Reverse all the bytes.
    // This un-reverses all of the reversals from Part 1.
    let bytes = unsafe {
        let s = s;
        // This is safe because `s` is &mut str so guaranteed not to be aliased.
        slice::from_raw_parts_mut(s.as_ptr() as *mut u8, s.len())
    };
    bytes.reverse();

    // Each UTF-8 sequence is now in the right order.
    debug_assert!(str::from_utf8(bytes).is_ok());
}

#[cfg(test)]
mod tests {
    use super::reverse_grapheme_clusters_in_place;

    extern crate std;
    use self::std::string::ToString;

    fn test_rev(a: &str, b: &str) {
        let mut a = a.to_string();
        reverse_grapheme_clusters_in_place(&mut a);
        assert_eq!(a, b);
    }

    #[test]
    fn test_empty() {
        test_rev("", "");
    }

    #[test]
    fn test_ascii() {
        test_rev("Hello", "olleH");
    }

    #[test]
    fn test_utf8() {
        test_rev("¡Hola!", "!aloH¡");
    }

    #[test]
    fn test_emoji() {
        test_rev("\u{1F36D}\u{1F36E}", "\u{1F36E}\u{1F36D}");
    }

    #[test]
    fn test_combining_mark() {
        test_rev("man\u{0303}ana", "anan\u{0303}am");
    }
}
