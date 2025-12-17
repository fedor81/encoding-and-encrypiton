use anyhow::Result;
use clap::{Parser, ValueEnum};
use pseudorandom::{LCG, PRNG, XorShift32, get_random_seed};
use strum::Display;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run()
}

impl Cli {
    fn run(&self) -> Result<()> {
        let mut generator = self.algorithm.get_generator(self.seed)?;
        (0..self.count).for_each(|_| println!("{}", generator.next()));
        Ok(())
    }
}

/// Генератор псевдослучайных чисел
#[derive(Parser, Debug)]
#[command(version, author = "laroxyss")]
struct Cli {
    /// Алгоритм генерации
    #[arg(short, long, default_value_t)]
    algorithm: Algorithm,

    /// Количество генерируемых чисел
    #[arg(short, long, default_value_t = 10)]
    count: usize,

    /// Зерно для генератора
    ///
    /// Если не задано, то будет использоваться случайно сгенерированное.
    #[arg(short, long, default_value_t = get_random_seed())]
    seed: u64,
}

#[derive(Debug, ValueEnum, Default, Clone, Copy, Display)]
#[value(rename_all = "lower")] // Отображение в clap как lowercase
#[strum(serialize_all = "lowercase")] // Отображение везде в программе как lowercase
enum Algorithm {
    /// Линейный конгруэнтный генератор - одна из старейших и простейших схем.
    /// Недостатки: низкое качество случайности (короткий период, корреляции).
    LCG,

    /// Быстрый, компактный, использует побитовые операции (XOR и сдвиги).
    /// Лучше LCG по качеству и скорости.
    #[default]
    XorShift,
}

impl Algorithm {
    fn get_generator(&self, seed: u64) -> Result<Box<dyn PRNG<Item = u32>>> {
        let seed_u32 = (seed % u32::MAX as u64) as u32;
        Ok(match self {
            Algorithm::LCG => Box::new(LCG::build(seed_u32)?),
            Algorithm::XorShift => Box::new(XorShift32::build(seed_u32)?),
        })
    }
}
