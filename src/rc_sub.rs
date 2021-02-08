//! Short self-contained module for showing off two primary solutions. The first
//! is quite simple but also with some good performance characteristics, and the
//! second is more verbose and uses a manual implementation of UTF8 encoding for
//! numbers, but is significantly faster.

pub fn main() {
    crackle_pop();
    // crackle_pop_fast();
}

/// Conservatively give more than enough byte space, so that we only need 1 allocation.
const CAPACITY: usize = "CracklePop".len() * 100;

fn crackle_pop() {
    let mut str = String::with_capacity(CAPACITY);
    for n in 1..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        if div_by_3 {
            str.push_str("Crackle");
        }
        if div_by_5 {
            str.push_str("Pop");
        }
        if !(div_by_3 || div_by_5) {
            str.push_str(&n.to_string());
        }
        str.push('\n');
    }
    print!("{}", str.trim());
}

/// About 7x faster than the simpler implementation.
#[allow(unused)]
fn crackle_pop_fast() {
    let mut buf = Vec::with_capacity(CAPACITY);
    const CRACKLE: &[u8] = b"Crackle";
    const POP: &[u8] = b"Pop";
    const CRACKLE_POP: &[u8] = b"CracklePop";

    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        if div_by_3 && div_by_5 {
            buf.extend_from_slice(CRACKLE_POP);
        } else if div_by_3 {
            buf.extend_from_slice(CRACKLE);
        } else if div_by_5 {
            buf.extend_from_slice(POP);
        } else {
            write_1_or_2_digit_u8_as_utf8(n, &mut buf);
        };
        buf.push(b'\n');
    }

    // Safe because of testing write_1_or_2_digit_u8_as_utf8 with 0 to 99.
    print!("{}", unsafe { String::from_utf8_unchecked(buf).trim() });
}

use std::io::Write;
/// Encodes a 1 or 2 digit u8 number in utf8 format (for general IO printing),
/// and writes it to a buffer.
fn write_1_or_2_digit_u8_as_utf8<W: Write>(x: u8, buf: &mut W) {
    debug_assert!(x < 100);
    const UTF8_ZERO: u8 = b'0';
    if x < 10 {
        buf.write_all(&[UTF8_ZERO + x]).unwrap();
    } else {
        let ones = x % 10;
        let tens = x / 10;
        buf.write_all(&[UTF8_ZERO + tens, UTF8_ZERO + ones])
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::*;
    use test::Bencher;

    #[test]
    fn write_1_or_2_digit_u8_as_utf8_yields_valid_utf8() {
        for n in 0..100 {
            let mut buf = Vec::new();
            write_1_or_2_digit_u8_as_utf8(n, &mut buf);
            assert_eq!(String::from_utf8_lossy(&buf), n.to_string());
            buf.clear();
        }
    }

    #[bench]
    // test rc_sub::tests::normal                                   ... bench:       4,414 ns/iter (+/- 216)
    fn normal(b: &mut Bencher) {
        b.iter(|| crackle_pop());
    }

    #[bench]
    // test rc_sub::tests::fast                                     ... bench:         618 ns/iter (+/- 88)
    fn fast(b: &mut Bencher) {
        b.iter(|| crackle_pop_fast());
    }
}
