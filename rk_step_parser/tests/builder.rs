use rk_step_parser::{parse_step_file, build_graph};

const STEP: &str = include_str!("fixtures/cube.step");

#[test]
fn graph_contains_cartesian_point() {
    let sf = parse_step_file(STEP).unwrap();
    let g  = build_graph(&sf.entities);

    // FreeCAD 立方体だと #12 が CARTESIAN_POINT
    let node = g.get(&12).expect("node #12 not found");

    assert_eq!(node.kind, "CARTESIAN_POINT");
    // 先頭引数は座標のリスト
    if let rk_step_parser::Attr::List(xyz) = &node.attrs[1] {
        assert_eq!(xyz.len(), 3);
    } else {
        panic!("unexpected attr format");
    }
}
