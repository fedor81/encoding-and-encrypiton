use std::ops::Index;

use anyhow::{Context, Result};

use super::{Canvas, Module};

impl Canvas {
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn new(size: usize) -> Self {
        Self {
            modules: vec![vec![Module::default(); size]; size],
        }
    }

    /// Попытаться установить модуль по координатам (x, y)
    pub fn try_set(&mut self, x: usize, y: usize, module: Module) -> Result<()> {
        self.try_set_with(x, y, || module)
    }

    /// Попытаться установить модуль, используя функцию
    pub fn try_set_with<F>(&mut self, x: usize, y: usize, f: F) -> Result<()>
    where
        F: FnOnce() -> Module,
    {
        self.check_bounds(x, y)?;
        self.modules[y][x]
            .try_set_with(f)
            .with_context(|| format!("Cannot set module at ({}, {})", x, y))
    }

    /// Проверить границы и вернуть ошибку с контекстом
    fn check_bounds(&self, x: usize, y: usize) -> Result<()> {
        if x >= self.len() || y >= self.len() {
            anyhow::bail!("Canvas index out of bounds: ({}, {}), size={}", x, y, self.len());
        }
        Ok(())
    }
}

impl Index<usize> for Canvas {
    type Output = Vec<Module>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.modules[index]
    }
}
