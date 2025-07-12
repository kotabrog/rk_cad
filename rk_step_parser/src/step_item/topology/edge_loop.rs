//! Representation of the STEP **edge_loop** entity (ISO 10303‑42).
//!
//! ENTITY edge_loop
//!   SUBTYPE OF (loop, path);
//! DERIVE
//!   ne : INTEGER := SIZEOF (SELF\path.edge_list);
//! WHERE
//!   WR1: (SELF\path.edge_list[1].edge_start) :=
//!        (SELF\path.edge_list[ne].edge_end);
//! END_ENTITY;
//!
//! ENTITY loop
//!   SUPERTYPE OF (ONEOF(vertex_loop, edge_loop, poly_loop))
//!   SUBTYPE OF (topological_representation_item);
//! END_ENTITY;
//!
//! ENTITY path
//!   SUPERTYPE OF (ONEOF(open_path, edge_loop, oriented_path))
//!   SUBTYPE OF (topological_representation_item);
//!   edge_list : LIST [1:?] OF UNIQUE oriented_edge;
//! WHERE
//!   WR1: path_head_to_tail (SELF);
//! END_ENTITY;
//!
//! 注意:
//! - パスの最初のエッジの始点と、最後のエッジの終点は同じ頂点でなければならない
//!   - 同一な頂点であることは、idで判断される
//! - パスの中の隣接するエッジの始点と終点は一致しなければならない
//!  - これも同一な頂点であることは、idで判断される
//! - edge_listは要素数が1以上
//! - 同じパス内で1本のエッジを参照できるのは1回だけ
//!   - ただし、同一性はoriented_edgeのidのみで判断することとする
//!     - 実際にどのように判断されているかはわかっていない
//!     - 以下のパターンは非公式命題の範囲にあると考えられるため、validationはしていない
//!       - 同一のedgeからoriented_edgeが作られている可能性
//!       - 別々のedgeであるが、本質的に同一なedgeである可能性

use super::super::common::{
    aggregate_to_reference, check_keyword, expect_attr_len, expect_single_item_cast,
    ConversionStepItemError, FromSimple, HasKeyword, StepItemCast,
};
use super::super::{OrientedEdge, StepItem, VertexPoint};
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item::ValidateRefs;
use crate::step_item_map::{StepItemMap, StepItems};
use rk_calc::Vector3;

#[derive(Debug, Clone)]
pub struct EdgeLoop {
    pub edge_list: Vec<EntityId>,
}

impl HasKeyword for EdgeLoop {
    const KEYWORD: &'static str = "EDGE_LOOP";
}

impl FromSimple for EdgeLoop {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have exactly 2 parameters (name, edge_list)
        expect_attr_len(&se, 2, Self::KEYWORD)?;

        // edge_list = list of references
        let edge_list = aggregate_to_reference(&se.attrs[1], Self::KEYWORD)?;

        Ok(Self { edge_list })
    }
}

impl ValidateRefs for EdgeLoop {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        // edge_list が空でないことを確認
        if self.edge_list.is_empty() {
            return Err(ConversionStepItemError::NonEmptyList {
                keyword: Self::KEYWORD,
            });
        }

        // 同一idがないかどうかの確認
        {
            let mut seen_edges = std::collections::HashSet::new();
            for edge_id in &self.edge_list {
                if !seen_edges.insert(*edge_id) {
                    return Err(ConversionStepItemError::DuplicateReference {
                        keyword: Self::KEYWORD,
                        id: *edge_id,
                    });
                }
            }
        }

        // 各エッジが oriented_edge であることを確認
        let mut oriented_edges = Vec::with_capacity(self.edge_list.len());
        for edge_id in &self.edge_list {
            let oriented_edge = expect_single_item_cast::<OrientedEdge>(arena, *edge_id)?;
            oriented_edges.push(oriented_edge);
        }

        // エッジごとの端点を取得
        let mut start_points = Vec::with_capacity(oriented_edges.len());
        let mut end_points = Vec::with_capacity(oriented_edges.len());
        for oriented_edge in &oriented_edges {
            let (start_point, end_point) = oriented_edge.edge_start_and_end(arena)?;
            start_points.push(start_point);
            end_points.push(end_point);
        }

        // 最初のエッジの始点と最後のエッジの終点が同じであることを確認
        if start_points.first() != end_points.last() {
            return Err(ConversionStepItemError::LoopStartEndMismatch {
                keyword: Self::KEYWORD,
                start: start_points.first().cloned().unwrap_or_default(),
                end: end_points.last().cloned().unwrap_or_default(),
            });
        }

        // ループ内の隣り合うエッジの始点と終点が一致することを確認
        for i in 0..oriented_edges.len() - 1 {
            if end_points[i] != start_points[i + 1] {
                return Err(ConversionStepItemError::PathNotConnected {
                    keyword: Self::KEYWORD,
                    start: start_points[i + 1],
                    end: end_points[i],
                });
            }
        }
        Ok(())
    }
}

impl StepItemCast for EdgeLoop {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::EdgeLoop(el) => Some(el),
            _ => None,
        }
    }
}
impl From<EdgeLoop> for StepItem {
    fn from(el: EdgeLoop) -> Self {
        StepItem::EdgeLoop(Box::new(el))
    }
}

impl EdgeLoop {
    // 各値から arena に StepItem を登録するクラスメソッド
    pub fn new(edge_list: Vec<EntityId>) -> Self {
        Self { edge_list }
    }

    pub fn new_and_register(edge_list: Vec<EntityId>, arena: &mut StepItemMap) -> EntityId {
        let edge_loop = Self::new(edge_list);
        arena.insert_default_id(StepItems::new_with_one_item(edge_loop.into()))
    }

    pub fn register_step_item_map_line_default_loop(
        points: Vec<Vector3>,
        arena: &mut StepItemMap,
    ) -> EntityId {
        let mut edge_list = Vec::with_capacity(points.len());
        for i in 0..points.len() {
            let start_point = VertexPoint::register_step_item_map(points[i], arena);
            let end_point =
                VertexPoint::register_step_item_map(points[(i + 1) % points.len()], arena);
            let oriented_edge = OrientedEdge::register_step_item_map_line_default(
                start_point,
                end_point,
                true,
                arena,
            )
            .unwrap();
            edge_list.push(oriented_edge);
        }
        Self::new_and_register(edge_list, arena)
    }
}

#[cfg(test)]
mod tests {
    use super::super::VertexPoint;
    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item_map::StepItemMap;
    use rk_calc::Vector3;

    #[test]
    fn test_edge_loop_from_simple() {
        let se = SimpleEntity {
            keyword: "EDGE_LOOP".to_string(),
            attrs: vec![
                Parameter::String("oriented_edge_1".to_string()),
                Parameter::Aggregate(vec![
                    Parameter::Reference(1),
                    Parameter::Reference(2),
                    Parameter::Reference(3),
                ]),
            ],
        };
        let edge_loop = EdgeLoop::from_simple(se).unwrap();
        assert_eq!(edge_loop.edge_list.len(), 3);
        assert_eq!(edge_loop.edge_list[0], 1);
        assert_eq!(edge_loop.edge_list[1], 2);
        assert_eq!(edge_loop.edge_list[2], 3);
    }

    #[test]
    fn test_edge_loop_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID_KEYWORD".to_string(),
            attrs: vec![
                Parameter::String("oriented_edge_1".to_string()),
                Parameter::Aggregate(vec![Parameter::Reference(1), Parameter::Reference(2)]),
            ],
        };
        let result = EdgeLoop::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::Unsupported(_)
        ));
    }

    #[test]
    fn test_edge_loop_from_simple_invalid_attr_len() {
        let se = SimpleEntity {
            keyword: "EDGE_LOOP".to_string(),
            attrs: vec![Parameter::String("oriented_edge_1".to_string())],
        };
        let result = EdgeLoop::from_simple(se);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::AttrCount { expected, found, keyword } if expected == 2 && found == 1 && keyword == "EDGE_LOOP")
        );
    }

    #[test]
    fn test_edge_loop_from_simple_not_aggregate() {
        let se = SimpleEntity {
            keyword: "EDGE_LOOP".to_string(),
            attrs: vec![
                Parameter::String("oriented_edge_1".to_string()),
                Parameter::Reference(1), // Not an aggregate
            ],
        };
        let result = EdgeLoop::from_simple(se);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::NotAggregate { keyword }
            if keyword == "EDGE_LOOP")
        );
    }

    #[test]
    fn test_edge_loop_from_simple_not_reference_aggregate() {
        let se = SimpleEntity {
            keyword: "EDGE_LOOP".to_string(),
            attrs: vec![
                Parameter::String("oriented_edge_1".to_string()),
                Parameter::Aggregate(vec![Parameter::String("not_a_reference".to_string())]),
            ],
        };
        let result = EdgeLoop::from_simple(se);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::NotReference { keyword }
            if keyword == "EDGE_LOOP")
        );
    }

    #[test]
    fn test_edge_loop_validate_refs() {
        let mut arena = StepItemMap::new();

        let point0 = VertexPoint::register_step_item_map(Vector3::new(0.0, 0.0, 0.0), &mut arena);
        let point1 = VertexPoint::register_step_item_map(Vector3::new(1.0, 0.0, 0.0), &mut arena);
        let point2 = VertexPoint::register_step_item_map(Vector3::new(1.0, 1.0, 0.0), &mut arena);
        let point3 = VertexPoint::register_step_item_map(Vector3::new(0.0, 1.0, 0.0), &mut arena);

        let oriented_edge1 =
            OrientedEdge::register_step_item_map_line_default(point0, point1, true, &mut arena)
                .unwrap();
        let oriented_edge2 =
            OrientedEdge::register_step_item_map_line_default(point2, point1, false, &mut arena)
                .unwrap();
        let oriented_edge3 =
            OrientedEdge::register_step_item_map_line_default(point2, point3, true, &mut arena)
                .unwrap();
        let oriented_edge4 =
            OrientedEdge::register_step_item_map_line_default(point3, point0, true, &mut arena)
                .unwrap();

        let edge_loop = EdgeLoop::new(vec![
            oriented_edge1,
            oriented_edge2,
            oriented_edge3,
            oriented_edge4,
        ]);
        let result = edge_loop.validate_refs(&arena);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_loop_validate_refs_empty_edge_list() {
        let arena = StepItemMap::new();
        let edge_loop = EdgeLoop::new(vec![]);
        let result = edge_loop.validate_refs(&arena);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::NonEmptyList { keyword } if keyword == "EDGE_LOOP")
        );
    }

    #[test]
    fn test_edge_loop_validate_refs_duplicate_edges() {
        let mut arena = StepItemMap::new();

        let point0 = VertexPoint::register_step_item_map(Vector3::new(0.0, 0.0, 0.0), &mut arena);
        let point1 = VertexPoint::register_step_item_map(Vector3::new(1.0, 0.0, 0.0), &mut arena);

        let oriented_edge1 =
            OrientedEdge::register_step_item_map_line_default(point0, point1, true, &mut arena)
                .unwrap();
        let oriented_edge2 =
            OrientedEdge::register_step_item_map_line_default(point1, point0, false, &mut arena)
                .unwrap();

        let edge_loop = EdgeLoop::new(vec![oriented_edge1, oriented_edge2, oriented_edge1]);
        let result = edge_loop.validate_refs(&arena);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::DuplicateReference { keyword, id } if keyword == "EDGE_LOOP" && id == oriented_edge1)
        );
    }

    #[test]
    fn test_edge_loop_validate_refs_start_end_mismatch() {
        let mut arena = StepItemMap::new();

        let point0 = VertexPoint::register_step_item_map(Vector3::new(0.0, 0.0, 0.0), &mut arena);
        let point1 = VertexPoint::register_step_item_map(Vector3::new(1.0, 0.0, 0.0), &mut arena);
        let point2 = VertexPoint::register_step_item_map(Vector3::new(1.0, 1.0, 0.0), &mut arena);

        let oriented_edge1 =
            OrientedEdge::register_step_item_map_line_default(point0, point1, true, &mut arena)
                .unwrap();
        let oriented_edge2 =
            OrientedEdge::register_step_item_map_line_default(point1, point2, true, &mut arena)
                .unwrap();

        let edge_loop = EdgeLoop::new(vec![oriented_edge1, oriented_edge2]);
        let result = edge_loop.validate_refs(&arena);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::LoopStartEndMismatch { keyword, start, end } if keyword == "EDGE_LOOP" && start == point0 && end == point2)
        );
    }

    #[test]
    fn test_edge_loop_validate_refs_path_not_connected() {
        let mut arena = StepItemMap::new();

        let point0 = VertexPoint::register_step_item_map(Vector3::new(0.0, 0.0, 0.0), &mut arena);
        let point1 = VertexPoint::register_step_item_map(Vector3::new(1.0, 0.0, 0.0), &mut arena);
        let point2 = VertexPoint::register_step_item_map(Vector3::new(1.0, 1.0, 0.0), &mut arena);

        let oriented_edge1 =
            OrientedEdge::register_step_item_map_line_default(point0, point1, true, &mut arena)
                .unwrap();
        let oriented_edge2 =
            OrientedEdge::register_step_item_map_line_default(point2, point0, true, &mut arena)
                .unwrap();

        let edge_loop = EdgeLoop::new(vec![oriented_edge1, oriented_edge2]);
        let result = edge_loop.validate_refs(&arena);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::PathNotConnected { keyword, start, end } if keyword == "EDGE_LOOP" && start == point2 && end == point1)
        );
    }
}
