use std::path::PathBuf;

use crate::StateSaver;

pub trait Decoder {}

pub trait FileDecoder
where
    Self: Decoder + StateSaver + Sized,
{
    fn decode_file(self, target: &PathBuf, destination: &PathBuf) {
        todo!()
    }
}
