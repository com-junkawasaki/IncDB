//! Temporal Compression
//!
//! 時系列圧縮（Delta Encoding + Product Quantization + 重複排除）

use crate::compression::product_quantization::{ProductQuantization, QuantizedVector, PQError};
use incdb_core::model::{Event, Value};
use std::collections::HashSet;
use thiserror::Error;

/// 時系列圧縮エラー
#[derive(Error, Debug)]
pub enum CompressionError {
    #[error("PQ error: {0}")]
    PQ(#[from] PQError),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// 圧縮されたイベント
#[derive(Clone, Debug)]
pub struct CompressedEvents {
    /// タイムスタンプの差分エンコーディング
    pub timestamp_deltas: Vec<i64>,
    /// 量子化されたベクトル
    pub quantized_vectors: Vec<QuantizedVector>,
    /// 圧縮されたイベントデータ（bincode）
    pub compressed_data: Vec<u8>,
    /// イベントID
    pub event_ids: Vec<u64>,
    /// ベースタイムスタンプ（最初のイベントのタイムスタンプ）
    pub base_timestamp: i64,
}

/// 時系列圧縮
pub struct TemporalCompression {
    /// Delta Encodingを有効にするか
    delta_encoding: bool,
    /// ベクトル量子化
    vector_quantization: Option<ProductQuantization>,
    /// 重複排除を有効にするか
    deduplication: bool,
}

impl TemporalCompression {
    /// 新しい時系列圧縮を作成
    pub fn new(
        delta_encoding: bool,
        vector_quant: Option<ProductQuantization>,
        deduplication: bool,
    ) -> Self {
        Self {
            delta_encoding,
            vector_quantization: vector_quant,
            deduplication,
        }
    }

    /// イベントを圧縮
    ///
    /// # Arguments
    /// * `events` - 圧縮するイベント（時系列順）
    ///
    /// # Returns
    /// 圧縮されたイベント
    pub fn compress_events(&self, events: Vec<Event>) -> Result<CompressedEvents, CompressionError> {
        if events.is_empty() {
            return Ok(CompressedEvents {
                timestamp_deltas: Vec::new(),
                quantized_vectors: Vec::new(),
                compressed_data: Vec::new(),
                event_ids: Vec::new(),
                base_timestamp: 0,
            });
        }

        // 重複排除
        let events = if self.deduplication {
            self.deduplicate_events(events)
        } else {
            events
        };

        // ベースタイムスタンプ（最初のイベントのタイムスタンプ）
        let base_timestamp = events[0].timestamp;

        // タイムスタンプの差分エンコーディング
        let timestamp_deltas: Vec<i64> = if self.delta_encoding {
            events
                .iter()
                .enumerate()
                .map(|(i, event)| {
                    if i == 0 {
                        0 // 最初のイベントは差分0
                    } else {
                        event.timestamp - events[i - 1].timestamp
                    }
                })
                .collect()
        } else {
            events.iter().map(|e| e.timestamp - base_timestamp).collect()
        };

        // ベクトル量子化
        let quantized_vectors: Vec<QuantizedVector> = if let Some(ref pq) = self.vector_quantization {
            events
                .iter()
                .filter_map(|event| {
                    event.embedding.as_ref().and_then(|embedding| {
                        pq.quantize(embedding).ok()
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        // イベントデータを圧縮（embeddingを除外）
        let mut events_without_embedding: Vec<Event> = events
            .iter()
            .map(|event| {
                let mut e = event.clone();
                e.embedding = None; // 埋め込みは量子化されたものを使用
                e
            })
            .collect();

        // bincodeでシリアライズ
        let compressed_data = bincode::serialize(&events_without_embedding)
            .map_err(|e| CompressionError::Serialization(e.to_string()))?;

        // イベントIDを保存
        let event_ids: Vec<u64> = events.iter().map(|e| e.id.0).collect();

        Ok(CompressedEvents {
            timestamp_deltas,
            quantized_vectors,
            compressed_data,
            event_ids,
            base_timestamp,
        })
    }

    /// 圧縮データを展開
    ///
    /// # Arguments
    /// * `compressed` - 圧縮されたイベント
    ///
    /// # Returns
    /// 展開されたイベント
    pub fn decompress_events(
        &self,
        compressed: CompressedEvents,
    ) -> Result<Vec<Event>, CompressionError> {
        // イベントデータをデシリアライズ
        let mut events: Vec<Event> = bincode::deserialize(&compressed.compressed_data)
            .map_err(|e| CompressionError::Serialization(e.to_string()))?;

        // タイムスタンプを復元
        if self.delta_encoding {
            let mut current_timestamp = compressed.base_timestamp;
            for (i, delta) in compressed.timestamp_deltas.iter().enumerate() {
                if i < events.len() {
                    current_timestamp += delta;
                    events[i].timestamp = current_timestamp;
                }
            }
        } else {
            for (i, event) in events.iter_mut().enumerate() {
                if i < compressed.timestamp_deltas.len() {
                    event.timestamp = compressed.base_timestamp + compressed.timestamp_deltas[i];
                }
            }
        }

        // ベクトルを復元
        if let Some(ref pq) = self.vector_quantization {
            let mut quantized_idx = 0;
            for event in &mut events {
                if event.embedding.is_none() && quantized_idx < compressed.quantized_vectors.len() {
                    let quantized = &compressed.quantized_vectors[quantized_idx];
                    if let Ok(reconstructed) = pq.dequantize(quantized) {
                        event.embedding = Some(reconstructed);
                        quantized_idx += 1;
                    }
                }
            }
        }

        // イベントIDを復元
        for (i, event) in events.iter_mut().enumerate() {
            if i < compressed.event_ids.len() {
                use incdb_core::model::IId;
                event.id = IId(compressed.event_ids[i]);
            }
        }

        Ok(events)
    }

    /// イベントの重複排除
    ///
    /// 同じIDのイベントは最初のもののみを保持
    fn deduplicate_events(&self, events: Vec<Event>) -> Vec<Event> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        for event in events {
            if seen.insert(event.id) {
                result.push(event);
            }
        }

        result
    }

    /// 圧縮率を計算
    ///
    /// # Arguments
    /// * `original_size` - 元のサイズ（バイト）
    /// * `compressed_size` - 圧縮後のサイズ（バイト）
    ///
    /// # Returns
    /// 圧縮率（0.0-1.0、小さいほど圧縮率が高い）
    pub fn compression_ratio(original_size: usize, compressed_size: usize) -> f64 {
        if original_size == 0 {
            return 0.0;
        }
        compressed_size as f64 / original_size as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use incdb_core::model::{Event, EventType, IId, Value};

    #[test]
    fn test_temporal_compression() {
        let pq = ProductQuantization::new(128, 8, 256).ok();
        let compression = TemporalCompression::new(true, pq, false);

        let mut events = Vec::new();
        for i in 0..10 {
            let embedding: Vec<f32> = (0..128).map(|j| (j as f32 + i as f32) / 128.0).collect();
            let event = Event::new(
                IId(i as u64 + 1),
                1000 + i * 100,
                EventType::PersonCreated,
                IId(1),
                Value::string(format!("Event {}", i)),
            )
            .with_embedding(embedding);
            events.push(event);
        }

        let compressed = compression.compress_events(events.clone()).unwrap();
        assert_eq!(compressed.event_ids.len(), 10);
        assert_eq!(compressed.timestamp_deltas.len(), 10);

        let decompressed = compression.decompress_events(compressed).unwrap();
        assert_eq!(decompressed.len(), 10);
    }

    #[test]
    fn test_deduplication() {
        let compression = TemporalCompression::new(true, None, true);

        let mut events = Vec::new();
        for i in 0..5 {
            let event = Event::new(
                IId(1), // 同じID
                1000 + i * 100,
                EventType::PersonCreated,
                IId(1),
                Value::string("Duplicate"),
            );
            events.push(event);
        }

        let compressed = compression.compress_events(events).unwrap();
        let decompressed = compression.decompress_events(compressed).unwrap();
        assert_eq!(decompressed.len(), 1); // 重複が排除される
    }
}

