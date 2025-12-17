//! pseudorandom crate - PRNG (Pseudo-Random Number Generators)
//!
//! # Описание
//!
//! `pseudorandom` — это библиотека, реализующая генераторы псевдослучайных чисел (PRNG) на Rust.

mod lcg;
mod prng_ext;
mod random_seed;
mod xorshift;

pub use lcg::LCG;
pub use prng_ext::PRNGExt;
pub use random_seed::get_random_seed;
pub use xorshift::XorShift32;

/// Генераторы псевдослучайных чисел (PRNG — Pseudo-Random Number Generators) — это алгоритмы,
/// которые генерируют последовательности чисел, кажущихся случайными, но полностью определяемых
/// начальным значением — семенем (seed).
pub trait PRNG {
    type Item;

    fn next(&mut self) -> Self::Item;
}
