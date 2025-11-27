use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

use qr_code_generator::{
    Drawable,
    barcode::code128::{Code128, CodeSet},
    qrcode::CorrectionLevel,
};
use strum::Display;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.command.execute()
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
        args: AppArgs,

        /// Стартовый используемый набор символов. Набор можно переключать внутри input используя специальные символы: À Ɓ Ć
        #[arg(short, long, default_value_t, ignore_case = true)]
        codeset: AppBarcodeSet,
    },

    /// Команда для построения QR кода.
    QR {
        #[command(flatten)]
        args: AppArgs,

        /// Уровень коррекции ошибок.
        #[arg(short, long, default_value_t, ignore_case = true)]
        ecc: AppQRCorrLevel,
        // ignore_case = true — чтобы не было конфликта strum::Display
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

#[derive(ValueEnum, Copy, Clone, Default, Debug, Display)]
#[value(rename_all = "UPPERCASE")]
enum AppBarcodeSet {
    /// Символы с кодами 0–95: A-Z, 0-9, специальные символы и FNC 1-4
    A,

    #[default]
    /// ASCII символы с кодами 32–127: A-Z, a-z, 0-9, специальные символы и FNC 1-4
    B,

    /// Используется для парных цифр (00–99). Позволяет компактно кодировать числа: две цифры кодируются одним символом
    C,
}

impl From<AppBarcodeSet> for CodeSet {
    fn from(value: AppBarcodeSet) -> Self {
        match value {
            AppBarcodeSet::A => CodeSet::A,
            AppBarcodeSet::B => CodeSet::B,
            AppBarcodeSet::C => CodeSet::C,
        }
    }
}

impl Command {
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::Barcode { args, codeset } => Self::execute_barcode(&args.input, *codeset, &args.path),
            Self::QR { args, ecc } => Self::execute_qr(&args.input, *ecc, &args.path),
        }
    }

    fn execute_barcode<P: AsRef<Path>>(data: &str, barcodeset: AppBarcodeSet, path: P) -> Result<()> {
        let code = Code128::encode_with_codeset(&data, CodeSet::from(barcodeset))?;
        qr_code_generator::barcode::draw_barcode(&code, path)
    }

    fn execute_qr<P: AsRef<Path>>(data: &str, corr_level: AppQRCorrLevel, path: P) -> Result<()> {
        let qr = qr_code_generator::qrcode::QRCode::build_with_default_encoder(data.as_bytes(), corr_level.into())?;
        qr.draw(path)?;
        Ok(())
    }
}

#[derive(Debug, ValueEnum, Default, Clone, Copy, Display)]
enum AppQRCorrLevel {
    /// Низкий уровень коррекции ошибок (L).  
    /// Восстанавливает до 7% повреждённых данных.  
    /// Наименьший объём резервных данных — максимум данных на один размер матрицы.
    Low,

    /// Средний уровень коррекции ошибок (M).  
    /// Восстанавливает до 15% повреждённых данных.  
    /// Баланс между надёжностью и плотностью данных.  
    /// Значение по умолчанию.
    #[default]
    Medium,

    /// Уровень коррекции ошибок Q (Quartile).  
    /// Восстанавливает до 25% повреждённых данных.  
    /// Подходит для QR-кодов, которые могут быть частично повреждены (например, на упаковке).
    Quartile,

    /// Высокий уровень коррекции ошибок (H).  
    /// Восстанавливает до 30% повреждённых данных — максимальная надёжность.  
    /// Используется, когда QR-код может быть серьёзно повреждён (царапины, загрязнения).
    High,
}

impl Into<CorrectionLevel> for AppQRCorrLevel {
    fn into(self) -> CorrectionLevel {
        match self {
            Self::Low => CorrectionLevel::L,
            Self::Medium => CorrectionLevel::M,
            Self::Quartile => CorrectionLevel::Q,
            Self::High => CorrectionLevel::H,
        }
    }
}
