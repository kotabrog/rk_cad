use rk_step_parser::{build_graph, parse_step_file, Attr};

const STEP: &str = include_str!("fixtures/cube.step");

#[test]
fn graph_contains_cartesian_point() {
    let sf = parse_step_file(STEP).unwrap();
    let g = build_graph(&sf.entities);

    // どれか 1 つ CARTESIAN_POINT を拾う
    let node = g
        .values()
        .find(|n| n.kind == "CARTESIAN_POINT")
        .expect("no CARTESIAN_POINT found");

    // attrs は RefCell<Vec<Attr>>
    let attrs = node.attrs.borrow();

    // 第 2 引数が座標リストか確認
    if let Attr::List(xyz) = &attrs[1] {
        assert_eq!(xyz.len(), 3);
    } else {
        panic!("CARTESIAN_POINT arg[1] is not a List");
    }
}
