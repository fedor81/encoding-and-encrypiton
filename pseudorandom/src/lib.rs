mod lcg;
mod xorshift;

pub use xorshift::XorShift32;

/// Генераторы псевдослучайных чисел (PRNG — Pseudo-Random Number Generators) — это алгоритмы,
/// которые генерируют последовательности чисел, кажущихся случайными, но полностью определяемых
/// начальным значением — семенем (seed).
pub trait PRNG {
    type Item;

    fn new(seed: Self::Item) -> Self;
    fn next(&mut self) -> Self::Item;
}
