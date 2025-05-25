//! import_step.rs
//! STEP → Entity/Attr 変換フェーズ
//! ---------------------------------
//! 依存モジュール --------------------
use thiserror::Error;

use crate::{
    step_file::{parse_step_file, StepFileParseError},
    raw_entity::{parse_raw_entity, RawEntityParseError},
    entity_attr::{Entity, AttrParseError},
};

// ───────────────────────────────────
// エラー型（wrap して 1 つに統合）
// ───────────────────────────────────
#[derive(Debug, Error)]
pub enum ImportStepError {
    #[error(transparent)]
    StepFile(#[from] StepFileParseError),

    #[error(transparent)]
    RawEntity(#[from] RawEntityParseError),

    #[error(transparent)]
    Attr(#[from] AttrParseError),
}

// ───────────────────────────────────
// 公開 API
// ───────────────────────────────────
/// STEP ファイル文字列を受け取り、`Entity` ベクタを返す。
/// - HEADER／Trailer はいったん無視して DATA→Entity/Attr だけ生成
/// - 将来 CAD クラスへの変換は別フェーズで組み立てる
pub fn import_step(src: &str) -> Result<Vec<Entity>, ImportStepError> {
    // 文件全体を HEADER / DATA / Trailer に分離
    let step = parse_step_file(src)?;

    let mut entities = Vec::new();

    // DATA 行 → RawEntity → Entity(+Attr)
    for raw_line in &step.entities {
        // 行文字列 → Option<RawEntity>
        if let Some(raw) = parse_raw_entity(raw_line)? {
            // RawEntity → Entity(Attr)
            let entity = Entity::try_from(&raw)?;
            entities.push(entity);
        }
        // None の行はコメント・未対応行として無視
    }

    Ok(entities)
}
