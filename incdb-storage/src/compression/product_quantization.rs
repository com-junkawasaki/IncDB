//! Product Quantization
//!
//! ベクトル量子化による圧縮

use thiserror::Error;

/// Product Quantization エラー
#[derive(Error, Debug)]
pub enum PQError {
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("Empty codebook")]
    EmptyCodebook,
    #[error("Invalid subvector index: {index}")]
    InvalidSubvectorIndex { index: usize },
}

/// コードブック
///
/// 各サブベクトル空間の代表ベクトル集合
#[derive(Clone, Debug)]
pub struct Codebook {
    /// コードブックのベクトル（k個の代表ベクトル）
    vectors: Vec<Vec<f32>>,
    /// サブベクトルの次元数
    subvector_dim: usize,
}

impl Codebook {
    /// 新しいコードブックを作成
    ///
    /// # Arguments
    /// * `vectors` - k個の代表ベクトル
    /// * `subvector_dim` - サブベクトルの次元数
    pub fn new(vectors: Vec<Vec<f32>>, subvector_dim: usize) -> Result<Self, PQError> {
        if vectors.is_empty() {
            return Err(PQError::EmptyCodebook);
        }

        // すべてのベクトルが同じ次元を持つことを確認
        let expected_dim = subvector_dim;
        for (i, vec) in vectors.iter().enumerate() {
            if vec.len() != expected_dim {
                return Err(PQError::DimensionMismatch {
                    expected: expected_dim,
                    actual: vec.len(),
                });
            }
        }

        Ok(Self {
            vectors,
            subvector_dim,
        })
    }

    /// コードブックのサイズ（k）
    pub fn size(&self) -> usize {
        self.vectors.len()
    }

    /// サブベクトルの次元数
    pub fn subvector_dim(&self) -> usize {
        self.subvector_dim
    }

    /// インデックスでベクトルを取得
    pub fn get(&self, index: usize) -> Option<&[f32]> {
        self.vectors.get(index).map(|v| v.as_slice())
    }
}

/// 量子化されたベクトル
///
/// 各サブベクトルがコードブックのインデックスで表現される
#[derive(Clone, Debug)]
pub struct QuantizedVector {
    /// 各サブベクトルのコードブックインデックス（m個）
    codes: Vec<u8>,
    /// サブベクトル数（m）
    m: usize,
}

impl QuantizedVector {
    /// 新しい量子化ベクトルを作成
    pub fn new(codes: Vec<u8>, m: usize) -> Self {
        Self { codes, m }
    }

    /// コードを取得
    pub fn codes(&self) -> &[u8] {
        &self.codes
    }

    /// サブベクトル数
    pub fn m(&self) -> usize {
        self.m
    }

    /// サイズ（バイト数）
    pub fn size_bytes(&self) -> usize {
        self.codes.len()
    }
}

/// Product Quantization
///
/// ベクトルを複数のサブベクトルに分割し、各サブベクトルをコードブックで量子化
pub struct ProductQuantization {
    /// コードブック（各サブベクトル空間用）
    codebooks: Vec<Codebook>,
    /// サブベクトル数（m）
    m: usize,
    /// コードブックサイズ（k）
    k: usize,
    /// 元のベクトルの次元数
    original_dim: usize,
}

impl ProductQuantization {
    /// 新しいProduct Quantizationを作成
    ///
    /// # Arguments
    /// * `original_dim` - 元のベクトルの次元数（例: 128）
    /// * `m` - サブベクトル数（例: 8）
    /// * `k` - コードブックサイズ（例: 256）
    pub fn new(original_dim: usize, m: usize, k: usize) -> Result<Self, PQError> {
        if original_dim % m != 0 {
            return Err(PQError::DimensionMismatch {
                expected: original_dim,
                actual: original_dim,
            });
        }

        let subvector_dim = original_dim / m;

        // 各サブベクトル空間用のコードブックを初期化
        // 簡易実装: ランダムな代表ベクトルで初期化
        // 実際の実装では、k-meansなどで最適化されたコードブックを使用
        let mut codebooks = Vec::new();
        for _ in 0..m {
            let mut vectors = Vec::new();
            for _ in 0..k {
                let mut vec = Vec::with_capacity(subvector_dim);
                for _ in 0..subvector_dim {
                    vec.push(rand::random::<f32>() * 2.0 - 1.0); // -1.0 to 1.0
                }
                vectors.push(vec);
            }
            let codebook = Codebook::new(vectors, subvector_dim)?;
            codebooks.push(codebook);
        }

        Ok(Self {
            codebooks,
            m,
            k,
            original_dim,
        })
    }

    /// ベクトルを量子化（128次元 → 8 bytes）
    ///
    /// # Arguments
    /// * `vector` - 元のベクトル
    ///
    /// # Returns
    /// 量子化されたベクトル（m個のコード）
    pub fn quantize(&self, vector: &[f32]) -> Result<QuantizedVector, PQError> {
        if vector.len() != self.original_dim {
            return Err(PQError::DimensionMismatch {
                expected: self.original_dim,
                actual: vector.len(),
            });
        }

        let subvector_dim = self.original_dim / self.m;
        let mut codes = Vec::with_capacity(self.m);

        for i in 0..self.m {
            let start = i * subvector_dim;
            let end = start + subvector_dim;
            let subvector = &vector[start..end];

            // 最も近いコードブックベクトルを見つける
            let codebook = &self.codebooks[i];
            let mut best_index = 0;
            let mut best_distance = f32::MAX;

            for (idx, codebook_vec) in codebook.vectors.iter().enumerate() {
                let distance = Self::euclidean_distance(subvector, codebook_vec);
                if distance < best_distance {
                    best_distance = distance;
                    best_index = idx;
                }
            }

            codes.push(best_index as u8);
        }

        Ok(QuantizedVector::new(codes, self.m))
    }

    /// 量子化ベクトルを復元（近似）
    ///
    /// # Arguments
    /// * `quantized` - 量子化されたベクトル
    ///
    /// # Returns
    /// 復元されたベクトル（近似）
    pub fn dequantize(&self, quantized: &QuantizedVector) -> Result<Vec<f32>, PQError> {
        if quantized.m() != self.m {
            return Err(PQError::DimensionMismatch {
                expected: self.m,
                actual: quantized.m(),
            });
        }

        let subvector_dim = self.original_dim / self.m;
        let mut result = Vec::with_capacity(self.original_dim);

        for (i, &code) in quantized.codes().iter().enumerate() {
            if i >= self.codebooks.len() {
                return Err(PQError::InvalidSubvectorIndex { index: i });
            }

            let codebook = &self.codebooks[i];
            if let Some(subvector) = codebook.get(code as usize) {
                result.extend_from_slice(subvector);
            } else {
                return Err(PQError::InvalidSubvectorIndex { index: code as usize });
            }
        }

        Ok(result)
    }

    /// ユークリッド距離を計算
    fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum::<f32>()
            .sqrt()
    }

    /// サブベクトル数
    pub fn m(&self) -> usize {
        self.m
    }

    /// コードブックサイズ
    pub fn k(&self) -> usize {
        self.k
    }

    /// 元のベクトルの次元数
    pub fn original_dim(&self) -> usize {
        self.original_dim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_quantization() {
        let pq = ProductQuantization::new(128, 8, 256).unwrap();
        assert_eq!(pq.m(), 8);
        assert_eq!(pq.k(), 256);
        assert_eq!(pq.original_dim(), 128);

        // テストベクトル
        let vector: Vec<f32> = (0..128).map(|i| (i as f32) / 128.0).collect();

        // 量子化
        let quantized = pq.quantize(&vector).unwrap();
        assert_eq!(quantized.m(), 8);
        assert_eq!(quantized.size_bytes(), 8); // 8 bytes

        // 復元
        let reconstructed = pq.dequantize(&quantized).unwrap();
        assert_eq!(reconstructed.len(), 128);
    }
}

