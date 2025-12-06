//! Witness Structures
//!
//! 普遍性や構造の witness を保持する構造体

use crate::foundation::nno::NnoWitness;

/// Witness のコレクション
pub struct Witnesses {
    /// NNO の witness
    pub nno: Option<NnoWitness>,
}

impl Witnesses {
    /// 新しい witness コレクションを作成
    pub fn new() -> Self {
        Self { nno: None }
    }

    /// NNO witness を設定
    pub fn set_nno(&mut self, witness: NnoWitness) {
        self.nno = Some(witness);
    }
}

impl Default for Witnesses {
    fn default() -> Self {
        Self::new()
    }
}

