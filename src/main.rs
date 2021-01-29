#![allow(unused_imports, dead_code, soft_unstable)]
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![feature(test, array_value_iter)]

use std::io::{self, prelude::*};
use std::{array::IntoIter, char, convert::TryInto, ops::Index};
use std::{mem, ops::Deref};

pub fn main() {
    println!("{:?}", char::from_digit(11, 16).unwrap() as u8);
    println!("{:?}", char::from_digit(11, 20));
    println!("{:?}", char::from_digit(11, 10));
    let b = &mut [0; 8];
    '0'.encode_utf8(b);
    println!("{:?}", b);

    // io::stdout().write(&[).unwrap();
    // println!(
    //     "{:?}",
    //     i32::to_ne_bytes(909090)
    //         .iter()
    //         .map(|x| format!("{:08b}", x))
    //         .collect::<Vec<_>>()
    // );
    // println!("{:?}", "10".to_ascii_lowercase().as_bytes());
    // println!("{:?}", 'س');
    // println!("{:?}", mem::size_of_val(&'س'));
    // println!("{:?}", mem::size_of_val(&format_args!("{}", 10)));
}

const MAX_CAP: usize = "CracklePop".len();

fn crackle_pop() {
    let mut str = String::with_capacity(MAX_CAP);
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
            str = n.to_string();
        }

        println!("{}", str);
        str.clear();
    }
}

/// Uses u8's and hardcoded const values rather than string buffer manipulation.
fn crackle_pop_hardcoded() {
    use std::char;

    const CRACKLE: &str = "Crackle";
    const POP: &str = "Pop";
    const CRACKLE_POP: &str = "CracklePop";

    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        let str = if div_by_3 {
            CRACKLE
        } else if div_by_5 {
            POP
        } else if div_by_3 && div_by_5 {
            CRACKLE_POP
        } else {
            ""
        };

        if str.is_empty() {
            println!("{}", n);
        } else {
            println!("{}", str);
        }
    }
}

/// Idea: separate out the numbers that need to get converted to unicode, and
/// look into using SIMD operations to batch the numerical additions needed
/// together.
fn crackle_pop_split_up() {
    unimplemented!()
}

/// This data structure will go directly on the stack. It is only intended to be
/// written to and consumed. Optimal for smaller IO (otherwise we'd want
/// dynamic).
///
/// No methods will check that writing to the buffer won't overflow. Instead,
/// Rust will just panic.
#[derive(Debug)]
struct ArrayBuffer<T, const N: usize> {
    /// The current position that we may write to.
    pos: usize,
    buf: [T; N],
}

impl<T: Default + Copy, const N: usize> ArrayBuffer<T, N> {
    pub fn new() -> Self {
        ArrayBuffer {
            pos: 0,
            buf: [T::default(); N],
        }
    }

    fn push_buf(&mut self, buf: &[T]) {
        let len = buf.len();
        for i in 0..len {
            self.buf[i + self.pos] = buf[i];
        }
        self.pos += len;
    }
}

impl<T, const N: usize> ArrayBuffer<T, N> {
    pub fn from(arr: [T; N]) -> Self {
        ArrayBuffer { pos: 0, buf: arr }
    }

    pub fn push_fixed<const M: usize>(&mut self, buf: [T; M]) {
        let pos = self.pos;
        IntoIter::new(buf)
            .enumerate()
            .for_each(|(i, x)| self.buf[pos + i] = x);
        self.pos += M;
    }

    pub fn push(&mut self, val: T) {
        self.buf[self.pos] = val;
        self.pos += 1;
    }
}

/// The ArrayBuffer simply derefs to the underlying buffer. We intentionally do
/// not provide DerefMut, as our buffer relies upon continuous writing to the
/// end.
impl<T, const N: usize> Deref for ArrayBuffer<T, N> {
    type Target = [T; N];
    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

/// We simply don't handle possibility for overflow and panic instead. A full
/// write will always be attempted, and only a panic will prevent it.
impl<const N: usize> Write for ArrayBuffer<u8, N> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.push_buf(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Encodes a u8 number in utf8 format (for general IO printing), and writes it
/// to a buffer.
fn write_u8_as_utf8<W: Write>(x: u8, buf: &mut W) {
    if x < 10 {
        buf.write_all(&[b'0' + x]).unwrap();
    } else if x < 100 {
        let ones = x % 10;
        let tens = x / 10;
        buf.write_all(&[b'0' + tens, b'0' + ones]).unwrap();
    } else {
        // Not particularly optimized. Current estimate from benches is 20x
        // slower. Albeit, this branch will be avoided during the crackle_pop
        // routine.
        let s_buf = format!("{}", x);
        buf.write_all(s_buf.as_bytes()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    // use std::fmt::Write;
    use std::{borrow::Cow, io::Write};
    use test::Bencher;

    #[test]
    fn array_buffer_works() {
        use super::ArrayBuffer;
        let arr = [0u8; 100];
        let mut ab = ArrayBuffer::from(arr);

        ab.push(0);
        ab.push_fixed([1, 2, 3, 4, 5]);
        ab.push(99);

        assert_eq!(&ab[0..8], &[0, 1, 2, 3, 4, 5, 99, 0]);
    }

    #[test]
    fn write_u8_as_utf8_works() {
        let mut buf = Vec::new();

        super::write_u8_as_utf8(6, &mut buf);
        assert_eq!(&buf, "6".as_bytes());

        buf.clear();

        super::write_u8_as_utf8(81, &mut buf);
        assert_eq!(&buf, "81".as_bytes());

        buf.clear();

        super::write_u8_as_utf8(240, &mut buf);
        assert_eq!(&buf, "240".as_bytes());
    }

    #[bench]
    fn concat_vs_hardcoded_cow(b: &mut Bencher) {
        const CRACKLE: &str = "Crackle";
        const POP: &str = "Pop";
        const CRACKLE_POP: &str = "CracklePop";

        let mut vec = Vec::with_capacity(1000);

        b.iter(|| {
            for i in 0..100 {
                let str: Cow<'_, str> = if i % 3 == 0 {
                    CRACKLE.into()
                } else if (i + 1) % 3 == 0 {
                    POP.into()
                } else if (i + 2) % 3 == 0 {
                    CRACKLE_POP.into()
                } else {
                    unreachable!()
                };
                vec.push(str);
            }
            vec.clear();
        });
    }

    // Turns out this one is at least 4 times slowe than using Cow by itself,
    // and at least a further twice as slow as using the fully hardcoded, no Cow
    // version (8 times slower net).
    #[bench]
    fn concat_vs_hardcoded_concat(b: &mut Bencher) {
        const CRACKLE: &str = "Crackle";
        const POP: &str = "Pop";

        let mut vec = Vec::with_capacity(1000);

        b.iter(|| {
            for i in 0..100 {
                let str: Cow<'_, str> = if i % 3 == 0 {
                    CRACKLE.into()
                } else if (i + 1) % 3 == 0 {
                    POP.into()
                } else if (i + 2) % 3 == 0 {
                    [CRACKLE, POP].concat().into()
                } else {
                    unreachable!()
                };
                vec.push(str);
            }
            vec.clear();
        });
    }

    #[bench]
    fn concat_vs_hardcoded_hardcoded(b: &mut Bencher) {
        const CRACKLE: &str = "Crackle";
        const POP: &str = "Pop";
        const CRACKLE_POP: &str = "CracklePop";

        let mut vec = Vec::with_capacity(1000);

        b.iter(|| {
            for i in 0..100 {
                let str = if i % 3 == 0 {
                    CRACKLE
                } else if (i + 1) % 3 == 0 {
                    POP
                } else if (i + 2) % 3 == 0 {
                    CRACKLE_POP
                } else {
                    unreachable!()
                };
                vec.push(str);
            }
            vec.clear();
        });
    }

    #[bench]
    fn crackle_pop(b: &mut Bencher) {
        b.iter(|| super::crackle_pop());
    }

    #[bench]
    fn crackle_pop_hardcoded(b: &mut Bencher) {
        b.iter(|| super::crackle_pop_hardcoded());
    }

    #[bench]
    fn num_via_vec_write(b: &mut Bencher) {
        let mut vec = Vec::with_capacity(10000);
        b.iter(|| {
            vec.clear();
            for i in 0u8..100 {
                write!(vec, "{}", i).unwrap();
            }
        });
    }

    #[bench]
    fn num_via_str_write(b: &mut Bencher) {
        use std::fmt::Write;
        let mut buf = String::with_capacity(10000);
        b.iter(|| {
            buf.clear();
            for i in 0u8..100 {
                write!(buf, "{}", i).unwrap();
            }
        });
    }

    // #[bench]
    // fn num_via_str_write(b: &mut Bencher) {
    //     use std::fmt::Write;
    //     let mut buf = [0u8; 200];
    //     b.iter(|| {
    //         buf.iter_mut().for_each(|x| *x = 0);
    //         for i in 0 as u8..100 {
    //             if i < 10 {
    //                 buf[i] = i.to_ascii_lowercase()
    //             }
    //         }
    //     });
    // }

    #[bench]
    fn write_u8_lt_100(b: &mut Bencher) {
        let vec = &mut Vec::with_capacity(1000);
        b.iter(|| {
            for i in 0..100 {
                super::write_u8_as_utf8(i, vec);
            }
            vec.clear();
        });
    }

    // According to benchmarks this performs literally about 20 times worse than
    // when handling values beneath 100.
    #[bench]
    fn write_u8_gt_100(b: &mut Bencher) {
        let vec = &mut Vec::with_capacity(1000);
        b.iter(|| {
            for i in 100..200 {
                super::write_u8_as_utf8(i, vec);
            }
            vec.clear();
        });
    }
}
