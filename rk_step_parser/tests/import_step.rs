use rk_step_parser::import_step;
const STEP: &str = include_str!("fixtures/cube.step");

#[test]
fn test_import_step() {
    let entities = import_step(STEP).unwrap();
    println!("{:?}", entities);
    println!("entities: {}", entities.len());
    assert_eq!(entities.len(), 182);
}
