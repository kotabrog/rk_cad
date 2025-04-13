// rk_step_parser/src/lib.rs
//! STEPパーサ。STEPファイル（ISO-10303-21形式）のうち、
//! この例ではBLOCKエンティティのみをサポートする非常にシンプルな実装例です。

use regex::Regex;
use rk_cad::{Block, CadModel};
use rk_calc::Vector3;
use std::error::Error;

/// STEPファイルの文字列内容を解析し、CadModelを生成する。
/// 対応するSTEP形式は以下のようなBLOCK定義のみとする（シンプルな例）。
///
/// 例）  
///   #1 = BLOCK('Block1', (0.0, 0.0, 0.0), (1.0, 2.0, 3.0));
///
pub fn parse_step(content: &str) -> Result<CadModel, Box<dyn Error>> {
    let block_regex = Regex::new(
        r#"(?x)
        ^\s*                        # 先頭の空白を許容
        \#\d+\s*=\s*                # ID（#数字）と"="、内容は無視
        BLOCK\s*                    # リテラル "BLOCK"
        \(\s*'([^']+)'\s*,\s*        # シングルクォートで囲まれたブロック名（キャプチャ1）
        \(\s*([^,]+),\s*([^,]+),\s*([^,]+)\s*\)\s*,\s*  # origin: (x, y, z)（キャプチャ2-4）
        \(\s*([^,]+),\s*([^,]+),\s*([^,]+)\s*\)\s*      # dimensions: (dx, dy, dz)（キャプチャ5-7）
        \)\s*;                      # 終了
        \s*$
    "#,
    )?;

    let mut model = CadModel::new();

    for line in content.lines() {
        if let Some(cap) = block_regex.captures(line) {
            let name = cap.get(1).unwrap().as_str();
            let ox: f64 = cap.get(2).unwrap().as_str().trim().parse()?;
            let oy: f64 = cap.get(3).unwrap().as_str().trim().parse()?;
            let oz: f64 = cap.get(4).unwrap().as_str().trim().parse()?;
            let dx: f64 = cap.get(5).unwrap().as_str().trim().parse()?;
            let dy: f64 = cap.get(6).unwrap().as_str().trim().parse()?;
            let dz: f64 = cap.get(7).unwrap().as_str().trim().parse()?;

            let block = Block::new(name, Vector3::new(ox, oy, oz), Vector3::new(dx, dy, dz));
            model.add_block(block);
        }
    }
    Ok(model)
}

/// CadModelをSTEP形式の文字列に変換する。
/// ここでは、非常にシンプルなヘッダと、各Blockを1行ずつ出力する。
pub fn write_step(model: &CadModel) -> String {
    let mut output = String::new();

    // 簡単なヘッダ（実際のSTEP仕様とは異なり、最小限の情報のみ）
    output.push_str("ISO-10303-21;\nHEADER;\n/* Minimal header information */\nENDSEC;\nDATA;\n");

    // 各Blockを順に書き出す（簡易なIDカウンタを使用）
    let mut id = 1;
    for block in &model.blocks {
        let line = format!(
            "#{} = BLOCK('{}', ({}, {}, {}), ({}, {}, {}));\n",
            id,
            block.name,
            block.origin.x,
            block.origin.y,
            block.origin.z,
            block.dimensions.x,
            block.dimensions.y,
            block.dimensions.z
        );
        output.push_str(&line);
        id += 1;
    }
    output.push_str("ENDSEC;\nEND-ISO-10303-21;");
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_write() {
        let input = "#1 = BLOCK('Block1', (0.0, 0.0, 0.0), (1.0, 2.0, 3.0));\n";
        let model = parse_step(input).unwrap();
        assert_eq!(model.blocks.len(), 1);
        assert_eq!(model.blocks[0].name, "Block1");
        assert_eq!(model.blocks[0].dimensions, Vector3::new(1.0, 2.0, 3.0));

        let output = write_step(&model);
        // 書き出した内容にBLOCK定義行が含まれていることを確認
        assert!(output.contains("BLOCK('Block1'"));
    }
}
