//! Compression
//!
//! 時系列圧縮とベクトル量子化

pub mod temporal_compression;
pub mod product_quantization;

pub use temporal_compression::{TemporalCompression, CompressedEvents};
pub use product_quantization::{ProductQuantization, QuantizedVector, Codebook};

