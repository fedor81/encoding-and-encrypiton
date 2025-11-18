use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

use qr_code_generator::barcode::{Code128, CodeSet};

fn main() -> Result<()> {
    let cli = Cli::parse();
    Ok(())
}

#[derive(Parser)]
#[command(version, author = "laroxyss")]
/// Генератор штрих и QR кодов
struct Cli {
    /// Исполняемая команда
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Команда для построения штрих кода Code 128.
    Barcode {
        #[command(flatten)]
        input_args: AppArgs,

        /// Стартовый используемый набор символов. Набор можно переключать внутри input используя специальные символы: À Ɓ Ć
        #[arg(short, long, default_value_t)]
        codeset: BarcodeSet,
    },
}

#[derive(Args)]
struct AppArgs {
    /// Данные, которые надо закодировать
    input: String,

    /// Путь для сохранения изображения
    #[arg(short, long, default_value = "code.png")]
    path: PathBuf,
}

#[derive(ValueEnum, Clone, Default, Debug)]
#[value(rename_all = "UPPERCASE")]
enum BarcodeSet {
    /// Символы с кодами 0–95: A-Z, 0-9, специальные символы и FNC 1-4
    A,

    #[default]
    /// ASCII символы с кодами 32–127: A-Z, a-z, 0-9, специальные символы и FNC 1-4
    B,

    /// Используется для парных цифр (00–99). Позволяет компактно кодировать числа: две цифры кодируются одним символом
    C,
}

impl std::fmt::Display for BarcodeSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BarcodeSet::A => write!(f, "A"),
            BarcodeSet::B => write!(f, "B"),
            BarcodeSet::C => write!(f, "C"),
        }
    }
}
