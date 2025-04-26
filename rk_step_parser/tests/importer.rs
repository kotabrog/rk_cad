use rk_step_parser::{parse_step_file, build_graph, resolve_refs, import_cube};
const STEP: &str = include_str!("fixtures/cube.step");

#[test]
fn cube_roundtrip_counts() {
    let sf = parse_step_file(STEP).unwrap();
    let mut g = build_graph(&sf.entities);
    resolve_refs(&mut g);

    let m = import_cube(&g).unwrap();
    assert_eq!(m.vertices().count(), 8);
    assert_eq!(m.edges().count(),   12);
    assert_eq!(m.faces().count(),    6);
    assert_eq!(m.solids().count(),   1);
}
