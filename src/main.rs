#![allow(unused_imports, dead_code, soft_unstable)]
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![feature(test, array_value_iter)]

use std::{array::IntoIter, char, ops::Index};
use std::{
    io::{self, prelude::*},
    marker::PhantomData,
};
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

fn crackle_pop_efficient() {
    use std::char;

    const CRACKLE: &str = "Crackle";
    const POP: &str = "Pop";
    const CRACKLE_POP: &str = "CracklePop";

    // let num_str_buf = [char::default(); 2];
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
            // let tens = n / 10;
            // num_str_buf[0] = char::from_digit(tens, 10).unwrap();
            // let ones = n % 10;
            // num_str_buf[1] = char::from_digit(ones, 10).unwrap();
            // num_str_buf.jo
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

    // fn push_buf(&mut self, buf: &[T]) {
    //     let pos = self.pos;
    //     IntoIter::new(buf)
    //         .enumerate()
    //         .for_each(|(i, x)| self.buf[pos + i] = x);
    //     self.pos += M;
    // }

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

// impl<T, const N: usize> Write for ArrayBuffer<T, N> {
//     fn write(&mut self, buf: &[u8]) -> Result<usize> {
//         if self.pos + buf.len() >= N {
//             Ok(0) // C
//         }
//     }
// }

/// Encodes a u8 number in utf8 format (for general IO printing), and writes it to a buffer.
// fn write_u8_as_utf8(x: u8, buf: &mut [u8]) {
//     if x < 10 {
//         buf.write(&[b'0' + x]);
//     } else {
//         let ones = x % 10;
//         let tens = x / 10;
//         buf.write(&[b'0' + tens, b'0' + ones])
//     }
// }

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

    #[bench]
    fn concat_vs_hardcoded_concat(b: &mut Bencher) {
        const CRACKLE: &str = "Crackle";
        const POP: &str = "Pop";

        // let mut vec = Vec::with_capacity(1000);

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
                // vec.push(str);
            }
            // vec.clear();
        });
    }

    #[bench]
    fn concat_vs_hardcoded_hardcoded(b: &mut Bencher) {
        const CRACKLE: &str = "Crackle";
        const POP: &str = "Pop";
        const CRACKLE_POP: &str = "CracklePop";

        // let mut vec = Vec::with_capacity(1000);

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
                // vec.push(str);
            }
            // vec.clear();
        });
    }

    #[bench]
    fn crackle_pop(b: &mut Bencher) {
        b.iter(|| super::crackle_pop());
    }

    #[bench]
    fn crackle_pop_efficient(b: &mut Bencher) {
        b.iter(|| super::crackle_pop_efficient());
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
}
