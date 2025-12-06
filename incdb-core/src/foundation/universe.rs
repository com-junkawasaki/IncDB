//! Type Universe Hierarchy
//!
//! U₀, U₁, U₂, ... の階層化実装

use crate::model::Level;

/// Universe 階層
///
/// Type の自己参照によるパラドックスを防ぐため、階層を導入
pub struct UniverseHierarchy {
    /// 現在の最大 level
    max_level: Level,
}

impl UniverseHierarchy {
    /// 新しい階層を作成
    pub fn new() -> Self {
        Self {
            max_level: Level(0),
        }
    }

    /// Level が有効かチェック
    ///
    /// level(i) <= level(j) のときのみ i -> j の型付け参照を許す
    pub fn can_reference(&self, from: Level, to: Level) -> bool {
        from.le(to)
    }

    /// 次の level を取得
    pub fn next_level(&self) -> Level {
        self.max_level.next()
    }
}

impl Default for UniverseHierarchy {
    fn default() -> Self {
        Self::new()
    }
}

/// Universe trait（将来の型レベル制約用）
pub trait Universe {
    /// この Universe の level
    const LEVEL: Level;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universe_hierarchy() {
        let hierarchy = UniverseHierarchy::new();
        let level0 = Level::zero();
        let level1 = level0.next();

        assert!(hierarchy.can_reference(level0, level1));
        assert!(!hierarchy.can_reference(level1, level0));
    }
}

