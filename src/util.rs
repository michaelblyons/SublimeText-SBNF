use std::str::{from_utf8_unchecked, Chars};

// A fast way to convert an interval of iterators to a substring. Rust should
// at least have an easy way to get byte indices from Chars :(
pub fn str_from_iterators<'a>(
    string: &'a str,
    start: Chars<'a>,
    end: Chars<'a>,
) -> &'a str {
    // Convert start and end into byte offsets
    let bytes_start = string.len() - start.as_str().len();
    let bytes_end = string.len() - end.as_str().len();

    // SAFETY: As long as the iterators are from the string the byte offsets
    // will always be valid.
    unsafe { from_utf8_unchecked(&string.as_bytes()[bytes_start..bytes_end]) }
}
