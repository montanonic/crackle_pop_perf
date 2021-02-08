//! Here are the benchmark results:
//!
//! test tests::main_crackle_pop                               ... bench:       8,454 ns/iter (+/- 2,463)
//! test tests::main_crackle_pop_arraybuf_minimal_vars         ... bench:         635 ns/iter (+/- 127)
//! test tests::main_crackle_pop_arraybuf_with_newline_methods ... bench:         769 ns/iter (+/- 38)
//! test tests::main_crackle_pop_arraybuf_with_own_write_u8    ... bench:         748 ns/iter (+/- 256)
//! test tests::main_crackle_pop_arrbuf                        ... bench:         669 ns/iter (+/- 136)
//! test tests::main_crackle_pop_faster_utf8                   ... bench:       4,177 ns/iter (+/- 290)
//! test tests::main_crackle_pop_hardcoded                     ... bench:       4,438 ns/iter (+/- 272)
//!
//! So when I instead of using println! at the end, just return the data structure itself, I get
//! test tests::main_crackle_pop_arraybuf_minimal_vars         ... bench:         409 ns/iter (+/- 41)
//! test tests::main_crackle_pop_vec_minimal_vars              ... bench:         488 ns/iter (+/- 36)
//!
//! Those two are the fastest implementations I have. Notice how the Vec version is only just barely
//! slower. Going back to the version with println!, we get:
//! test tests::main_crackle_pop_arraybuf_minimal_vars         ... bench:         668 ns/iter (+/- 127)
//! test tests::main_crackle_pop_vec_minimal_vars              ... bench:         718 ns/iter (+/- 132)
//!
//! Again with the vec a bit slower, but barely. Our println! overhead is quite significant, accounting
//! for roughly 1/4 - 1/3 of the total time.
#![warn(missing_debug_implementations, rust_2018_idioms)]
#![feature(test, array_value_iter)]

mod rc_sub;

use std::array::IntoIter;
use std::io::{self, prelude::*};
use std::ops::Deref;
use std::str;

pub fn main() {
    rc_sub::main()
}

/// 512 bytes, just enough for this problem. Can also test benchmarks with
/// larger values to see if it affects actual CPU performance in any way.
///
/// UPDATE: It absolutely impacts perf, in a big way. Our arraybuf impls went
/// from ~700ns to ~2600ns by increasing the array buffer size from 512 bytes to
/// 8192 bytes. Be wary of the stack size! With 65536 bytes, 0x10_000, we went
/// up to around 18,000ns per iter!
///
/// UPDATE UPDATE: Hmm, it looks like placing the struct behind a box doesn't
/// actually change the performance hit here very much, going up to
/// 19,800ns/iter for the minimal vars version (from 18k ns). I think I ought to
/// look into the implementation of the buffer itself and understand how the
/// size could be degrading performance.
///
/// Would it just be the initial allocation of the value that kills performance?
/// I can quickly test that by upping the size of the vector in the other impl.
///
/// Answer: YES! The initial allocation is what kills perf; I also zeroed the
/// vector to the same size of 0x10_000 in another impl and it has 23000ns/iter
/// performance (yikes). From this observation we should be more keen on
/// re-using buffers, and also potentially invest in creating a dynamically
/// sized one (but of course, stack rather than heap allocated).
const ARRAY_BUFFER_SIZE: usize = 0x800;
/// Conservatively give more than enough byte space, so that we only need 1 allocation.
const CAPACITY: usize = "CracklePop".len() * 100;

pub fn crackle_pop() {
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

/// Uses u8's and hardcoded const values rather than string buffer manipulation.
pub fn crackle_pop_hardcoded() {
    const CRACKLE: &str = "Crackle";
    const POP: &str = "Pop";
    const CRACKLE_POP: &str = "CracklePop";

    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        let str = if div_by_3 && div_by_5 {
            CRACKLE_POP
        } else if div_by_3 {
            CRACKLE
        } else if div_by_5 {
            POP
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

/// Furthers the hardcoded implementation with a function that more optimally
/// encodes u8 numbers into UTF8 characters representing them. Does a separate
/// newline write however, which may degrade performance (consider making a
/// writeln method on ArrayBuffer).
///
/// This implementation also currently uses a vector, which means we do hit the
/// heap here. Perhaps worth benchmark comparing to exactly the same form but
/// with a stack-allocated buffer. I would also like to see a version of this
/// which only uses one final println call at the end.
pub fn crackle_pop_faster_utf8() {
    const CRACKLE: &str = "Crackle";
    const POP: &str = "Pop";
    const CRACKLE_POP: &str = "CracklePop";

    // I'd prefer to use stdout directly here, but then it won't play well with
    // tests (as it won't suppress output, unlike println!).
    let mut vec = Vec::with_capacity(CAPACITY);
    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        let str = if div_by_3 && div_by_5 {
            CRACKLE_POP
        } else if div_by_3 {
            CRACKLE
        } else if div_by_5 {
            POP
        } else {
            ""
        };

        if str.is_empty() {
            unsafe {
                write_u8_as_utf8(n, &mut vec);
                vec.write(b"\n").unwrap();
                print!("{}", str::from_utf8_unchecked(&vec));
                vec.clear();
            }
        } else {
            println!("{}", str);
        }
    }
}

/// Furthers the faster utf8 implementation with an array-buffer to collect the
/// data and follow with a single write to stdout.
pub fn crackle_pop_arrbuf() {
    const CRACKLE: &str = "Crackle";
    const POP: &str = "Pop";
    const CRACKLE_POP: &str = "CracklePop";

    let mut buf: ArrayBuffer<_, ARRAY_BUFFER_SIZE> = ArrayBuffer::new();
    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        let str = if div_by_3 && div_by_5 {
            CRACKLE_POP
        } else if div_by_3 {
            CRACKLE
        } else if div_by_5 {
            POP
        } else {
            ""
        };

        if str.is_empty() {
            write_u8_as_utf8(n, &mut buf);
            buf.push(b'\n');
        } else {
            buf.push_buf(str.as_bytes());
            buf.push(b'\n');
        }
    }

    buf.write_all_to_stdout().unwrap();
}

/// Furthers the arraybuf impl by using its own write u8 as utf8 implementation
/// that doesn't go through the Writer trait.
pub fn crackle_pop_arraybuf_with_own_write_u8() {
    const CRACKLE: &str = "Crackle";
    const POP: &str = "Pop";
    const CRACKLE_POP: &str = "CracklePop";

    let mut buf: ArrayBuffer<_, ARRAY_BUFFER_SIZE> = ArrayBuffer::new();
    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        let str = if div_by_3 && div_by_5 {
            CRACKLE_POP
        } else if div_by_3 {
            CRACKLE
        } else if div_by_5 {
            POP
        } else {
            ""
        };

        if str.is_empty() {
            buf.write_u8_as_utf8(n);
            buf.push(b'\n');
        } else {
            buf.push_buf(str.as_bytes());
            buf.push(b'\n');
        }
    }

    buf.write_all_to_stdout().unwrap();
}

/// Furthers the arraybuf and own_write_u8 impls by using ArrayBuffer's built-in
/// newline pushing methods.
pub fn crackle_pop_arraybuf_with_newline_methods() {
    const CRACKLE: &str = "Crackle";
    const POP: &str = "Pop";
    const CRACKLE_POP: &str = "CracklePop";

    let mut buf: ArrayBuffer<_, ARRAY_BUFFER_SIZE> = ArrayBuffer::new();
    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        let str = if div_by_3 && div_by_5 {
            CRACKLE_POP
        } else if div_by_3 {
            CRACKLE
        } else if div_by_5 {
            POP
        } else {
            ""
        };

        if str.is_empty() {
            buf.write_u8_as_utf8_with_newline(n);
        } else {
            buf.push_buf_line(str.as_bytes());
        }
    }

    buf.write_all_to_stdout().unwrap();
}

/// Takes all the arraybuf and utf8 speed advancements and further optimizes the
/// amount of data transformations happening by working with bytes instead of
/// &str the whole time, and writing directly to the buffer rather than storing
/// an intermediate "str" var.
pub fn crackle_pop_arraybuf_minimal_vars() {
    const CRACKLE: &[u8] = b"Crackle";
    const POP: &[u8] = b"Pop";
    const CRACKLE_POP: &[u8] = b"CracklePop";

    let mut buf: ArrayBuffer<u8, ARRAY_BUFFER_SIZE> = ArrayBuffer::new();
    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        if div_by_3 && div_by_5 {
            buf.push_buf_line(CRACKLE_POP);
        } else if div_by_3 {
            buf.push_buf_line(CRACKLE);
        } else if div_by_5 {
            buf.push_buf_line(POP);
        } else {
            buf.write_u8_as_utf8_with_newline(n)
        };
    }

    buf.write_all_to_stdout().unwrap();
}

/// Does not use an arraybuffer, but uses a vec. Now we can compare stack vs
/// heap allocated data performance. This is based off of the minimal vars impl.
pub fn crackle_pop_vec_minimal_vars() {
    const CRACKLE: &[u8] = b"Crackle";
    const POP: &[u8] = b"Pop";
    const CRACKLE_POP: &[u8] = b"CracklePop";

    let mut buf = Vec::with_capacity(ARRAY_BUFFER_SIZE);
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
            write_u8_as_utf8(n, &mut buf);
        };
        buf.push(b'\n');
    }

    println!("{}", unsafe { str::from_utf8_unchecked(&buf) });
}

/// Doesn't use print, and doesn't use internal allocation.
pub fn crackle_pop_ext_arraybuf_minimal_vars(buf: &mut ArrayBuffer<u8, ARRAY_BUFFER_SIZE>) {
    const CRACKLE: &[u8] = b"Crackle";
    const POP: &[u8] = b"Pop";
    const CRACKLE_POP: &[u8] = b"CracklePop";

    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        if div_by_3 && div_by_5 {
            buf.push_buf_line(CRACKLE_POP);
        } else if div_by_3 {
            buf.push_buf_line(CRACKLE);
        } else if div_by_5 {
            buf.push_buf_line(POP);
        } else {
            buf.write_u8_as_utf8_with_newline(n)
        };
    }
}

/// Like crackle_pop_ext_arraybuf_minimal_vars, but takes ownership of arraybuf
/// rather than a reference.
pub fn crackle_pop_ext_owned_arraybuf_minimal_vars(
    mut buf: ArrayBuffer<u8, ARRAY_BUFFER_SIZE>,
) -> ArrayBuffer<u8, ARRAY_BUFFER_SIZE> {
    const CRACKLE: &[u8] = b"Crackle";
    const POP: &[u8] = b"Pop";
    const CRACKLE_POP: &[u8] = b"CracklePop";

    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        if div_by_3 && div_by_5 {
            buf.push_buf_line(CRACKLE_POP);
        } else if div_by_3 {
            buf.push_buf_line(CRACKLE);
        } else if div_by_5 {
            buf.push_buf_line(POP);
        } else {
            buf.write_u8_as_utf8_with_newline(n)
        };
    }

    buf
}

/// Doesn't use print, and doesn't use internal allocation.
pub fn crackle_pop_ext_vec_minimal_vars(buf: &mut Vec<u8>) {
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
            write_u8_as_utf8(n, buf);
        };
        buf.push(b'\n');
    }
}

/// The fastest vec impl.
pub fn crackle_pop_fastest_vec(buf: &mut Vec<u8>) {
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
            write_1_or_2_digit_u8_as_utf8(n, buf);
        };
        buf.push(b'\n');
    }
}

/// The fastest ArrayBuffer impl.
pub fn crackle_pop_fastest_arraybuf(buf: &mut ArrayBuffer<u8, ARRAY_BUFFER_SIZE>) {
    const CRACKLE: &[u8] = b"Crackle";
    const POP: &[u8] = b"Pop";
    const CRACKLE_POP: &[u8] = b"CracklePop";

    for n in 1u8..=100 {
        let div_by_3 = n % 3 == 0;
        let div_by_5 = n % 5 == 0;

        if div_by_3 && div_by_5 {
            buf.push_buf_line(CRACKLE_POP);
        } else if div_by_3 {
            buf.push_buf_line(CRACKLE);
        } else if div_by_5 {
            buf.push_buf_line(POP);
        } else {
            write_1_or_2_digit_u8_as_utf8(n, buf);
            buf.push(b'\n');
        };
    }
}

/// Idea: separate out the numbers that need to get converted to unicode, and
/// look into using SIMD operations to batch the numerical additions needed
/// together.
fn _crackle_pop_split_up() {
    unimplemented!()
}

/// Encodes a u8 number in utf8 format (for general IO printing), and writes it
/// to a buffer.
fn write_u8_as_utf8<W: Write>(x: u8, buf: &mut W) {
    const UTF8_ZERO: u8 = b'0';
    if x < 10 {
        buf.write_all(&[UTF8_ZERO + x]).unwrap();
    } else if x < 100 {
        let ones = x % 10;
        let tens = x / 10;
        buf.write_all(&[UTF8_ZERO + tens, UTF8_ZERO + ones])
            .unwrap();
    } else {
        // Not particularly optimized. Current estimate from benches is 20x
        // slower. Albeit, this branch will be avoided during the crackle_pop
        // routine (but the perf hit of compiling with a branch will remain).
        let s_buf = format!("{}", x);
        buf.write_all(s_buf.as_bytes()).unwrap();
    }
}

/// Encodes a 1 or 2 digit u8 number in utf8 format (for general IO printing),
/// and writes it to a buffer.
fn write_1_or_2_digit_u8_as_utf8<W: Write>(x: u8, buf: &mut W) {
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

/// This data structure will go directly on the stack. It is only intended to be
/// written to and consumed. Optimal for smaller IO (otherwise we'd want
/// dynamic). Barebones and prone to panic-ing.
///
/// No methods will check that writing to the buffer won't overflow. Instead,
/// Rust will just panic.
///
/// This structure allocates up front in FULL. Be mindful to re-use it where
/// possible rather than creating any large buffers internal to funcs/methods.
/// Small buffers could be fine. TODO: implement a more dynamic array buffer!
#[derive(Debug, Clone)]
pub struct ArrayBuffer<T, const N: usize> {
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
    #[allow(dead_code)] // Currently used in tests.
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

    fn push(&mut self, val: T) {
        self.buf[self.pos] = val;
        self.pos += 1;
    }
}

impl<const N: usize> ArrayBuffer<u8, N> {
    /// Attempts to write the entire buffer to stdout. If it fails, the
    /// operation has to be repeated, as no state is saved internally to track
    /// what was last printed.
    ///
    /// I can probably update this to write out to any sink, and then write a
    /// mock stdout for testing to avoid the problem of clobbering the terminal
    /// with line info. But for now, I've opted for a print! oriented
    /// implementation.
    pub fn write_all_to_stdout(&mut self) -> io::Result<()> {
        // io::stdout().write_all(&self.buf[0..self.pos])?;
        let str = unsafe { str::from_utf8_unchecked(&self.buf[0..self.pos]) };
        print!("{}", str);
        self.pos = 0;
        Ok(())
    }

    /// Functions identically to pushing a value and then pushing a newline
    /// character code, but with potentially higher performance.
    pub fn push_line(&mut self, val: u8) {
        self.buf[self.pos] = val;
        self.buf[self.pos + 1] = b'\n';
        self.pos += 2;
    }

    /// Functions identically to pushing a buffer and then pushing a newline
    /// character code, but with potentially higher performance.
    pub fn push_buf_line(&mut self, buf: &[u8]) {
        let len = buf.len();
        for i in 0..len {
            self.buf[i + self.pos] = buf[i];
        }
        self.buf[len + self.pos] = b'\n';
        self.pos += len + 1;
    }

    /// A specialized version of this function, working directly through array
    /// buffer methods rather than the general Write trait. I'm curious about
    /// potential performance differences.
    fn write_u8_as_utf8(&mut self, x: u8) {
        const UTF8_ZERO: u8 = b'0';
        if x < 10 {
            self.push(UTF8_ZERO + x);
        } else if x < 100 {
            let ones = x % 10;
            let tens = x / 10;
            self.push_fixed([UTF8_ZERO + tens, UTF8_ZERO + ones]);
        } else {
            let s_buf = format!("{}", x);
            self.push_buf(s_buf.as_bytes());
        }
    }

    /// A further specialized version that rolls in adding a newline as well.
    fn write_u8_as_utf8_with_newline(&mut self, x: u8) {
        const UTF8_ZERO: u8 = b'0';
        if x < 10 {
            self.push_line(UTF8_ZERO + x);
        } else if x < 100 {
            let ones = x % 10;
            let tens = x / 10;
            self.push_fixed([UTF8_ZERO + tens, UTF8_ZERO + ones, b'\n']);
        } else {
            let s_buf = format!("{}\n", x);
            self.push_buf(s_buf.as_bytes());
        }
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

#[cfg(test)]
mod tests {
    extern crate test;
    use std::{borrow::Cow, io::Write};
    use test::Bencher;

    use crate::{ArrayBuffer, ARRAY_BUFFER_SIZE};

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
    fn main_crackle_pop(b: &mut Bencher) {
        b.iter(|| super::crackle_pop());
    }

    #[bench]
    fn main_crackle_pop_hardcoded(b: &mut Bencher) {
        b.iter(|| super::crackle_pop_hardcoded());
    }

    #[bench]
    fn main_crackle_pop_faster_utf8(b: &mut Bencher) {
        b.iter(|| super::crackle_pop_faster_utf8());
    }

    #[bench]
    fn main_crackle_pop_arrbuf(b: &mut Bencher) {
        b.iter(|| super::crackle_pop_arrbuf());
    }

    #[bench]
    fn main_crackle_pop_arraybuf_with_own_write_u8(b: &mut Bencher) {
        b.iter(|| super::crackle_pop_arraybuf_with_own_write_u8());
    }

    #[bench]
    fn main_crackle_pop_arraybuf_with_newline_methods(b: &mut Bencher) {
        b.iter(|| super::crackle_pop_arraybuf_with_newline_methods());
    }

    #[bench]
    fn main_crackle_pop_arraybuf_minimal_vars(b: &mut Bencher) {
        b.iter(|| super::crackle_pop_arraybuf_minimal_vars());
    }

    #[bench]
    fn main_crackle_pop_vec_minimal_vars(b: &mut Bencher) {
        b.iter(|| super::crackle_pop_vec_minimal_vars());
    }

    /*
    With 0x800 ARRAY_BUFFER_SIZE and the CracklePop loop changed to be up to u8::MAX rather than 100,
    we notice scarily similar performance characteristics between our data structures. The owned version
    is more expensive as it requires an explicit clone to function. But the arraybuf and vec function
    nearly identically outside of that. Fascinating! Remember, the key to these three tests is that they
    do not allocate internally, but instead receive a buffer to place their results in.

    test tests::main_crackle_pop_ext_arraybuf_minimal_vars       ... bench:       7,499 ns/iter (+/- 1,044)
    test tests::main_crackle_pop_ext_owned_arraybuf_minimal_vars ... bench:       7,606 ns/iter (+/- 1,819)
    test tests::main_crackle_pop_ext_vec_minimal_vars            ... bench:       7,437 ns/iter (+/- 1,584)

    With our normal loop size of 100, here's the results:
    test tests::main_crackle_pop_ext_arraybuf_minimal_vars       ... bench:         394 ns/iter (+/- 52)
    test tests::main_crackle_pop_ext_owned_arraybuf_minimal_vars ... bench:         461 ns/iter (+/- 32)
    test tests::main_crackle_pop_ext_vec_minimal_vars            ... bench:         420 ns/iter (+/- 25)

    So clearly working with 3-digit values is much more expensive. Some further testing can happen in this
    area. But in the end, the big performance gain wasn't arraybuf, it was deferring the write to println!
    until the very end. When we drop println! entirely from our computation, we have nearly identical
    performance characteristics!

    In other words... vec is very fast!

    It also means that write_u8_as_utf8 becomes a bottleneck once we get into 3-digit numbers. Normally,
    here's a rough look at the overhead that write_u8_as_utf8 adds (second test is with it commented out):

    test tests::main_crackle_pop_ext_vec_minimal_vars            ... bench:         420 ns/iter (+/- 25)
    test tests::main_crackle_pop_ext_vec_minimal_vars            ... bench:         242 ns/iter (+/- 10)

    And when we go up to u8::MAX, but without the write_u8 func:
    test tests::main_crackle_pop_ext_vec_minimal_vars            ... bench:         547 ns/iter (+/- 41)

    Compare this to
    test tests::main_crackle_pop_ext_vec_minimal_vars            ... bench:       7,437 ns/iter (+/- 1,584)
    and you see that the difference is wild! We're spending around 12x the time of the other parts of our
    computation just on 3-digit decoding.

    Now of course, for this particular problem, this isn't important in the slightest. Can we save any extra
    time by killing off our 3-digit branch?

    test tests::main_crackle_pop_ext_vec_minimal_vars            ... bench:         339 ns/iter (+/- 15)

    It looks like the answer is yes! Removing that branch gives another sizable percentage speedup!
    */

    #[bench]
    fn main_crackle_pop_ext_arraybuf_minimal_vars(b: &mut Bencher) {
        let mut buf: ArrayBuffer<u8, ARRAY_BUFFER_SIZE> = ArrayBuffer::new();
        b.iter(|| {
            super::crackle_pop_ext_arraybuf_minimal_vars(&mut buf);
            buf.pos = 0;
        });
    }

    #[bench]
    fn main_crackle_pop_ext_owned_arraybuf_minimal_vars(b: &mut Bencher) {
        let mut buf: ArrayBuffer<u8, ARRAY_BUFFER_SIZE> = ArrayBuffer::new();
        b.iter(|| {
            buf = super::crackle_pop_ext_owned_arraybuf_minimal_vars(buf.clone());
            buf.pos = 0;
        });
    }

    #[bench]
    fn main_crackle_pop_ext_vec_minimal_vars(b: &mut Bencher) {
        let mut buf = Vec::with_capacity(ARRAY_BUFFER_SIZE);
        b.iter(|| {
            super::crackle_pop_ext_vec_minimal_vars(&mut buf);
            buf.clear();
        });
    }

    /*
    test tests::main_crackle_pop_fastest_arraybuf                ... bench:         318 ns/iter (+/- 19)
    test tests::main_crackle_pop_fastest_vec                     ... bench:         340 ns/iter (+/- 13)

    test tests::main_crackle_pop_fastest_arraybuf                ... bench:         335 ns/iter (+/- 19)
    test tests::main_crackle_pop_fastest_vec                     ... bench:         339 ns/iter (+/- 9)

    In the end there seems to be a slight edge for arraybuf, but barely. And it could also just be the
    benefit of rolling in the newline calls into the same call.
    */

    #[bench]
    fn main_crackle_pop_fastest_vec(b: &mut Bencher) {
        let mut buf = Vec::with_capacity(ARRAY_BUFFER_SIZE);
        b.iter(|| {
            super::crackle_pop_fastest_vec(&mut buf);
            buf.clear();
        });
    }

    #[bench]
    fn main_crackle_pop_fastest_arraybuf(b: &mut Bencher) {
        let mut buf: ArrayBuffer<u8, ARRAY_BUFFER_SIZE> = ArrayBuffer::new();
        b.iter(|| {
            super::crackle_pop_fastest_arraybuf(&mut buf);
            buf.pos = 0;
        });
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

    /// This test shows that writing directly to stdout is not captured in tests
    /// unlike println! is...
    #[test]
    #[ignore]
    fn write_to_stdout() {
        let mut out = std::io::stdout();
        write!(out, "this is a test!!").unwrap();
    }
}
