//! Value Types
//!
//! Incidence が持つ値の型システム

use serde::{Deserialize, Serialize};

/// Incidence が持つ値
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// 文字列
    Str(String),
    /// 整数
    Int(i64),
    /// 浮動小数点数
    Float(f64),
    /// ブール値
    Bool(bool),
    /// リスト
    List(Vec<Value>),
    /// ベクトル（埋め込み）
    Vector(Vec<f32>),
    /// Null
    Null,
}

impl Value {
    /// 文字列から Value を作成
    pub fn string(s: impl Into<String>) -> Self {
        Self::Str(s.into())
    }

    /// 整数から Value を作成
    pub fn int(i: i64) -> Self {
        Self::Int(i)
    }

    /// 浮動小数点数から Value を作成
    pub fn float(f: f64) -> Self {
        Self::Float(f)
    }

    /// ブール値から Value を作成
    pub fn bool(b: bool) -> Self {
        Self::Bool(b)
    }

    /// ベクトルから Value を作成
    pub fn vector(v: Vec<f32>) -> Self {
        Self::Vector(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let v1 = Value::string("hello");
        let v2 = Value::int(42);
        let v3 = Value::float(3.14);
        let v4 = Value::bool(true);
        let v5 = Value::vector(vec![0.1, 0.2, 0.3]);

        assert_eq!(v1, Value::Str("hello".to_string()));
        assert_eq!(v2, Value::Int(42));
        assert_eq!(v3, Value::Float(3.14));
        assert_eq!(v4, Value::Bool(true));
        assert_eq!(v5, Value::Vector(vec![0.1, 0.2, 0.3]));
    }
}

