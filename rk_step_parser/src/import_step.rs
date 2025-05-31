//! import_step.rs
//! STEP → Entity/Attr 変換フェーズ
//! ---------------------------------
//! 依存モジュール --------------------
use thiserror::Error;

use crate::{
    step_entity::{parse_step_entity, StepEntityParseError},
    step_file::{parse_step_file, StepFileParseError},
    step_item_map::{to_step_item_map, StepItemMap, StepItemMapError},
};

// ───────────────────────────────────
// エラー型（wrap して 1 つに統合）
// ───────────────────────────────────
#[derive(Debug, Error)]
pub enum ImportStepError {
    #[error(transparent)]
    FileParse(#[from] StepFileParseError),
    #[error(transparent)]
    EntityParse(#[from] StepEntityParseError),
    #[error(transparent)]
    ItemMap(#[from] StepItemMapError),
}

// ───────────────────────────────────
// 公開 API
// ───────────────────────────────────
/// STEP ファイル文字列を受け取り、`StepEntity` ベクタを返す。
/// - HEADER／Trailer はいったん無視して DATA→StepEntity だけ生成
/// - 将来 CAD クラスへの変換は別フェーズで組み立てる
pub fn import_step(src: &str) -> Result<StepItemMap, ImportStepError> {
    // 文件全体を HEADER / DATA / Trailer に分離
    let step = parse_step_file(src)?;

    let mut entities = Vec::new();

    // DATA 行 → Entity(+Attr)
    for line in &step.entities {
        let trimmed = line.trim();

        // 空行・コメント行はスキップ
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // 1 行を StepEntity(AST) へパース
        let ast = parse_step_entity(trimmed)?;
        entities.push(ast);
    }

    let step_item_map = to_step_item_map(entities)?;

    Ok(step_item_map)
}
