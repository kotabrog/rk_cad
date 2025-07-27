const CUBE_STEP: &str = include_str!("fixtures/cube.step");

#[cfg(test)]
mod old_tests {
    use super::*;
    use rk_step_parser::old::step_file::parse_step_file;
    use rk_step_parser::write_step_file;

    #[test]
    fn roundtrip_cube() {
        let src = CUBE_STEP;
        let step = parse_step_file(src).unwrap();

        let mut out = Vec::new();
        write_step_file(&step, &mut out).unwrap();
        let out_str = String::from_utf8(out).unwrap();

        // FreeCAD 用にヘッダ先頭が残っているか
        assert!(out_str.starts_with("ISO-10303-21;"));
        // エンティティ数が保持されているか
        assert_eq!(
            step.entities.len(),
            parse_step_file(&out_str).unwrap().entities.len()
        );
    }
}

use rk_step_parser::step_file::{export_step_file, parse_step_file};

#[test]
fn roundtrip_cube() {
    let src = CUBE_STEP;
    let step = parse_step_file(src).unwrap();

    let dst = export_step_file(&step).unwrap();

    // src と dst が同じ内容か
    // ただし、すべての空白や改行の違いは無視する
    let src_normalized: String = src.chars().filter(|c| !c.is_whitespace()).collect();
    let dst_normalized: String = dst.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(src_normalized, dst_normalized);
}
