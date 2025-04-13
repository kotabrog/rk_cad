use regex::Regex;
use std::error::Error;

use rk_calc::Vector3;
use rk_cad::{Block, CadModel};

/// FreeCADで出力された立方体 STEP ファイルからジオメトリ情報を抽出し、
/// 抽出した CARTESIAN_POINT 値からバウンディングボックスを作成して Block として返す。
///
/// この実装は非常に簡易なもので、STEPファイル内のすべての CARTESIAN_POINT 行を探し出し、
/// そこから得られる座標の最小／最大値で境界ボックスを計算します。
pub fn parse_step(content: &str) -> Result<CadModel, Box<dyn Error>> {
    // CARTESIAN_POINT 行を正規表現でキャプチャする
    // 行頭に "#" 番号、"="、"CARTESIAN_POINT" の記述を仮定し、
    // 第二引数の座標情報をキャプチャする。
    let re = Regex::new(
        r#"(?m)^#\d+\s*=\s*CARTESIAN_POINT\(\s*'[^']*'\s*,\s*\(\s*([^)]*)\s*\)\s*\)\s*;"#
    )?;

    let mut points: Vec<(f64, f64, f64)> = Vec::new();

    // 各キャプチャ結果から座標文字列を取り出し、カンマで分割して f64 に変換
    for cap in re.captures_iter(content) {
        let coords_str = cap.get(1).unwrap().as_str();
        let coords: Vec<f64> = coords_str
            .split(',')
            .map(|s| s.trim())
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();
        if coords.len() >= 3 {
            points.push((coords[0], coords[1], coords[2]));
        }
    }

    if points.is_empty() {
        return Err("No CARTESIAN_POINT found in STEP file".into());
    }

    // 最小／最大座標を計算
    let (mut min_x, mut min_y, mut min_z) = (points[0].0, points[0].1, points[0].2);
    let (mut max_x, mut max_y, mut max_z) = (points[0].0, points[0].1, points[0].2);
    for &(x, y, z) in &points {
        if x < min_x { min_x = x; }
        if y < min_y { min_y = y; }
        if z < min_z { min_z = z; }
        if x > max_x { max_x = x; }
        if y > max_y { max_y = y; }
        if z > max_z { max_z = z; }
    }

    // バウンディングボックスの原点と寸法
    let origin = Vector3::new(min_x, min_y, min_z);
    let dims = Vector3::new(max_x - min_x, max_y - min_y, max_z - min_z);

    // 立方体として、名前は STEP ファイル中の PRODUCT の名称「立方体」を仮定
    let block = Block::new("立方体", origin, dims);
    let mut model = CadModel::new();
    model.add_block(block);
    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// tests/data/cube.step に配置された実際の STEP ファイルを読み込み、パース結果の検証を行うテスト。
    #[test]
    fn test_parse_cube_from_file() {
        // Cargo.toml と同じディレクトリを基準に、tests/data/cube.step のパスを作成
        let file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("data")
            .join("cube.step");

        // ファイル内容を文字列として読み込む
        let content = fs::read_to_string(&file_path)
            .expect(&format!("Failed to read STEP file at {:?}", file_path));

        // parse_step 関数でパース
        let model = parse_step(&content).expect("Failed to parse STEP file");

        // 立方体として Block が1個得られ、バウンディングボックスが原点(0,0,0)～(10,10,10)となっていることを検証
        assert_eq!(model.blocks.len(), 1);
        let block = &model.blocks[0];
        assert_eq!(block.name, "立方体");
        assert!((block.origin.x - 0.0).abs() < 1e-6);
        assert!((block.origin.y - 0.0).abs() < 1e-6);
        assert!((block.origin.z - 0.0).abs() < 1e-6);
        assert!((block.dimensions.x - 10.0).abs() < 1e-6);
        assert!((block.dimensions.y - 10.0).abs() < 1e-6);
        assert!((block.dimensions.z - 10.0).abs() < 1e-6);
    }
}
