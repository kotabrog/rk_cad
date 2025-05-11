use rk_step_parser::old::step_file::parse_step_file;
use rk_step_parser::{build_graph, import_cube, resolve_refs};
const STEP: &str = include_str!("fixtures/cube.step");

#[test]
fn cube_counts() {
    let sf = parse_step_file(STEP).unwrap();
    let g = build_graph(&sf.entities);
    resolve_refs(&g);

    let m = import_cube(&g).unwrap();
    assert_eq!(m.vertices().count(), 8);
    assert_eq!(m.edges().count(), 12);
    assert_eq!(m.faces().count(), 6);
    assert_eq!(m.solids().count(), 1);
}
