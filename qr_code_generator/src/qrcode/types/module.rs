use anyhow::Result;
use std::ops::Not;

use super::Module;

impl Not for Module {
    type Output = Self;
    fn not(self) -> Self {
        match self {
            Self::Light => Self::Dark,
            Self::Dark => Self::Light,
            Self::Unused => panic!("Unused module cannot be inverted"),
        }
    }
}

impl Module {
    pub fn is_dark(self) -> bool {
        self == Self::Dark
    }

    pub fn is_light(self) -> bool {
        self == Self::Light
    }

    pub fn is_unused(self) -> bool {
        self == Self::Unused
    }

    /// Устанавливает значение модуля, если он Unused, иначе ошибка
    pub fn try_set(&mut self, module: Module) -> Result<()> {
        self.try_set_with(|| module)
    }

    /// Устанавливает значение модуля, возвращаемой функцией. НО если модуль не Unused - ошибка
    pub fn try_set_with<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce() -> Module,
    {
        if !self.is_unused() {
            anyhow::bail!("Cannot replace a non-unused module with another module");
        }
        *self = f();
        Ok(())
    }
}
