use rk_step_parser::old::step_file::parse_step_file;
use rk_step_parser::{build_graph, export_model, import_cube, resolve_refs, write_step_file};

const STEP: &str = include_str!("fixtures/cube.step");

#[test]
fn cube_roundtrip() {
    /* 1. 解析 → Model */
    let sf = parse_step_file(STEP).unwrap();
    let g = build_graph(&sf.entities);
    resolve_refs(&g);
    let model = import_cube(&g).unwrap();

    /* 2. モデル → 新しい STEP */
    let out_sf = export_model(&model);
    let mut buf = Vec::new();
    write_step_file(&out_sf, &mut buf).unwrap();
    let out_str = String::from_utf8(buf).unwrap();

    let sf2 = parse_step_file(&out_str).unwrap();
    let g2 = build_graph(&sf2.entities);
    resolve_refs(&g2);
    let model2 = import_cube(&g2).unwrap();
    assert_eq!(model.vertices().count(), model2.vertices().count());
    assert_eq!(model.edges().count(), model2.edges().count());
    assert_eq!(model.faces().count(), model2.faces().count());
    assert_eq!(model.solids().count(), model2.solids().count());
}
