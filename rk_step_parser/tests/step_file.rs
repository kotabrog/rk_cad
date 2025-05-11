use rk_step_parser::old::step_file::parse_step_file;
use rk_step_parser::write_step_file;

const CUBE_STEP: &str = include_str!("fixtures/cube.step");

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
