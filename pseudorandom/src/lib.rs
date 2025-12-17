use std::{
    hash::{Hash, Hasher},
    process,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

mod lcg;
mod xorshift;

pub use lcg::LCG;
pub use xorshift::XorShift32;

/// Генераторы псевдослучайных чисел (PRNG — Pseudo-Random Number Generators) — это алгоритмы,
/// которые генерируют последовательности чисел, кажущихся случайными, но полностью определяемых
/// начальным значением — семенем (seed).
pub trait PRNG {
    type Item;

    fn next(&mut self) -> Self::Item;
}

/// Статический счётчик для разнообразия генерации seed
static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Генерирует уникальный seed на основе:
/// - PID процесса
/// - Текущего времени (в наносекундах)
/// - Счётчика вызовов
/// - Адреса статической переменной (COUNTER)
pub fn get_random_seed() -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    // PID
    let pid = process::id() as u64;
    pid.hash(&mut hasher);

    // Текущее время в наносекундах
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    now.as_nanos().hash(&mut hasher);

    // Счётчик вызовов (потокобезопасно)
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    counter.hash(&mut hasher);

    // Добавим ещё что-то "случайное" — адрес самой переменной COUNTER
    (&COUNTER as *const AtomicU64 as u64).hash(&mut hasher);

    hasher.finish()
}
