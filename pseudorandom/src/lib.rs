mod xorshift;

pub use xorshift::XorShift32;

/// Генераторы псевдослучайных чисел (PRNG — Pseudo-Random Number Generators) — это алгоритмы,
/// которые генерируют последовательности чисел, кажущихся случайными, но полностью определяемых
/// начальным значением — семенем (seed).
pub trait PRNG {
    fn new(seed: u32) -> Self;
    fn next(&mut self) -> u32;
}
