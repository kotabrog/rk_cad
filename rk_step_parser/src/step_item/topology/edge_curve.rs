//! Representation of the STEP **edge_curve** entity (ISO 10303‑42).
//!
//! ENTITY edge_curve
//!   SUBTYPE OF (edge, geometric_representation_item);
//!   edge_geometry : curve;
//!   same_sense    : BOOLEAN;
//! END_ENTITY;
//!
//! ENTITY edge
//!   SUPERTYPE OF (ONEOF (edge_curve, oriented_edge, subedge))
//!   SUBTYPE OF (topological_representation_item);
//!   edge_start : vertex;
//!   edge_end   : vertex;
//! END_ENTITY;
//!
//! ENTITY vertex
//!   SUBTYPE OF (topological_representation_item);
//! END_ENTITY;
//!
//! 注意：
//! - `edge_geometry` は、本来は `curve` 型の参照であるため、
//!   curve エンティティまたはそのすべての下位型を取れるが、
//!   現在は`LINE` のみを受け入れる。
//! - エッジの長さ（領域）は有限かつゼロではない
//! - edge_start と edge_end は、一般には同一点でも許容されるが、LINE の場合は、エッジの長さがゼロとなるため、許容されない
//!   - line上の点を pnt + u * dir の形で表現した場合の u の値を基準に確かめるが、0かどうかの許容値はGLOBAL_UNCERTAINTY_ASSIGNED_CONTEXTによって定義されるが、現在は暫定的に1.E-07としている
//! - edge_start と edge_end は vertexを受け入れるが、 vertex は vertex_point である必要がある
//! - 頂点はLINE上にある必要がある
//!   - LINE上にあるかどうかの許容差は、GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXTによって定義されるが、現在は暫定的に1.E-07としている
//! - same_sense は、エッジの方向と、curve の方向（LINE の場合はdirの方向）を一致させるかどうかを示す
//!   - 実態とsame_senseの値が食い違う場合は、STEPファイルの不整合となる

use super::super::common::{
    boolean_to_bool, check_keyword, expect_attr_len, expect_reference, expect_single_item_cast,
    ConversionStepItemError, FromSimple, HasKeyword, StepItemCast,
};
use super::super::{Line, StepItem};
use super::VertexPoint;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item::ValidateRefs;
use crate::step_item_map::{StepItemMap, StepItems};
use rk_calc::Vector3;

#[derive(Debug, Clone)]
pub struct EdgeCurve {
    pub edge_start: EntityId,    // Vertex
    pub edge_end: EntityId,      // Vertex
    pub edge_geometry: EntityId, // Curve (currently only LINE)
    pub same_sense: bool,        // BOOLEAN
}

impl HasKeyword for EdgeCurve {
    const KEYWORD: &'static str = "EDGE_CURVE";
}

impl FromSimple for EdgeCurve {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have exactly 4 parameters (name, edge_start, edge_end, edge_geometry, same_sense).
        expect_attr_len(&se, 5, Self::KEYWORD)?;

        // edge_start = #id
        let edge_start = expect_reference(&se.attrs[1], Self::KEYWORD)?;

        // edge_end = #id
        let edge_end = expect_reference(&se.attrs[2], Self::KEYWORD)?;

        // edge_geometry = #id
        let edge_geometry = expect_reference(&se.attrs[3], Self::KEYWORD)?;

        // same_sense = true/false
        let same_sense = boolean_to_bool(&se.attrs[4], Self::KEYWORD)?;

        Ok(EdgeCurve {
            edge_start,
            edge_end,
            edge_geometry,
            same_sense,
        })
    }
}

impl ValidateRefs for EdgeCurve {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        let edge_start_item = expect_single_item_cast::<VertexPoint>(arena, self.edge_start)?;
        let edge_end_item = expect_single_item_cast::<VertexPoint>(arena, self.edge_end)?;
        let edge_geometry_item = expect_single_item_cast::<Line>(arena, self.edge_geometry)?;

        let start = edge_start_item.vertex_geometry_value(arena)?;
        let end = edge_end_item.vertex_geometry_value(arena)?;

        // ライン上にあるかどうかを確認
        if !edge_geometry_item.contains_point(&start, arena)? {
            return Err(ConversionStepItemError::PointNotOnEdge {
                keyword: Self::KEYWORD,
                point: self.edge_start,
                id: self.edge_geometry,
            });
        }
        if !edge_geometry_item.contains_point(&end, arena)? {
            return Err(ConversionStepItemError::PointNotOnEdge {
                keyword: Self::KEYWORD,
                point: self.edge_end,
                id: self.edge_geometry,
            });
        }

        // 許容差を暫定的に 1e-7 とする
        let eps = 1e-7;

        let start_u = edge_geometry_item.u_value(&start, arena)?;
        let end_u = edge_geometry_item.u_value(&end, arena)?;

        // edge 長がゼロでないことを確認
        let line_dir_magnitude = edge_geometry_item.dir_magnitude_value(arena)?;
        if (start_u - end_u).abs() * line_dir_magnitude < eps {
            return Err(ConversionStepItemError::ZeroLength {
                keyword: Self::KEYWORD,
            });
        }

        // same_sense の値と実際の方向が一致するか確認
        if (start_u < end_u) != self.same_sense {
            return Err(ConversionStepItemError::SameSenseMismatch {
                keyword: Self::KEYWORD,
                same_sense: self.same_sense,
            });
        }
        Ok(())
    }
}

impl StepItemCast for EdgeCurve {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::EdgeCurve(boxed) => Some(boxed),
            _ => None,
        }
    }
}

impl From<EdgeCurve> for StepItem {
    fn from(edge_curve: EdgeCurve) -> Self {
        StepItem::EdgeCurve(Box::new(edge_curve))
    }
}

impl EdgeCurve {
    /// 各値から arena にStepItem を登録するクラスメソッド
    pub fn register_step_item_map(
        start_coord: Vector3,
        end_coord: Vector3,
        line_pnt_coord: Vector3,
        line_dir: Vector3,
        line_magnitude: f64,
        same_sense: bool,
        arena: &mut StepItemMap,
    ) -> EntityId {
        let start_vertex_id = VertexPoint::register_step_item_map(start_coord, arena);
        let end_vertex_id = VertexPoint::register_step_item_map(end_coord, arena);
        let line_id = Line::register_step_item_map(line_pnt_coord, line_dir, line_magnitude, arena);
        let edge_curve = EdgeCurve {
            edge_start: start_vertex_id,
            edge_end: end_vertex_id,
            edge_geometry: line_id,
            same_sense,
        };
        arena.insert_default_id(StepItems::new_with_one_item(edge_curve.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_entity::Parameter;

    #[test]
    fn test_edge_curve_from_simple() {
        let se = SimpleEntity {
            keyword: "EDGE_CURVE".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Reference(1),
                Parameter::Reference(2),
                Parameter::Reference(3),
                Parameter::Logical(Some(true)),
            ],
        };

        let edge_curve = EdgeCurve::from_simple(se).unwrap();
        assert_eq!(edge_curve.edge_start, 1);
        assert_eq!(edge_curve.edge_end, 2);
        assert_eq!(edge_curve.edge_geometry, 3);
        assert!(edge_curve.same_sense);
    }

    #[test]
    fn test_edge_curve_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Reference(1),
                Parameter::Reference(2),
                Parameter::Reference(3),
                Parameter::Logical(Some(true)),
            ],
        };

        let result = EdgeCurve::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::Unsupported(_)
        ));
    }

    #[test]
    fn test_edge_curve_from_simple_invalid_attr_len() {
        let se = SimpleEntity {
            keyword: "EDGE_CURVE".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Reference(1),
                Parameter::Reference(2),
            ],
        };

        let result = EdgeCurve::from_simple(se);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::AttrCount { expected, found, keyword } if expected == 5 && found == 3 && keyword == "EDGE_CURVE")
        );
    }

    #[test]
    fn test_edge_curve_from_simple_not_reference() {
        let se = SimpleEntity {
            keyword: "EDGE_CURVE".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Real(1.0), // Not a reference
                Parameter::Reference(2),
                Parameter::Reference(3),
                Parameter::Logical(Some(true)),
            ],
        };

        let result = EdgeCurve::from_simple(se);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::NotReference { keyword, .. } if keyword == "EDGE_CURVE")
        );
    }

    #[test]
    fn test_edge_curve_validate_refs() {
        let mut arena = StepItemMap::new();
        let edge_curve_id = EdgeCurve::register_step_item_map(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(1.0, 1.0, 1.0),
            1.0, // Magnitude of the direction vector
            true,
            &mut arena,
        );
        assert!(arena.contains_key(&edge_curve_id));
        let edge_curve = expect_single_item_cast::<EdgeCurve>(&arena, edge_curve_id).unwrap();
        let result = edge_curve.validate_refs(&arena);
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_curve_validate_refs_point_not_on_edge() {
        let mut arena = StepItemMap::new();
        let edge_curve_id = EdgeCurve::register_step_item_map(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 2.0),
            1.0,
            true,
            &mut arena,
        );
        let edge_curve = expect_single_item_cast::<EdgeCurve>(&arena, edge_curve_id).unwrap();
        let result = edge_curve.validate_refs(&arena);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::PointNotOnEdge { keyword, point, id } if keyword == "EDGE_CURVE" && point == edge_curve.edge_end && id == edge_curve.edge_geometry)
        );
    }

    #[test]
    fn test_edge_curve_validate_refs_zero_length() {
        let mut arena = StepItemMap::new();
        let edge_curve_id = EdgeCurve::register_step_item_map(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0), // Same point for start and end
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            true,
            &mut arena,
        );
        let edge_curve = expect_single_item_cast::<EdgeCurve>(&arena, edge_curve_id).unwrap();
        let result = edge_curve.validate_refs(&arena);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::ZeroLength { keyword } if keyword == "EDGE_CURVE")
        );
    }

    #[test]
    fn test_edge_curve_validate_refs_same_sense_mismatch() {
        let mut arena = StepItemMap::new();
        let edge_curve_id = EdgeCurve::register_step_item_map(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(-1.0, -1.0, -1.0),
            1.0,
            true,
            &mut arena,
        );
        let edge_curve = expect_single_item_cast::<EdgeCurve>(&arena, edge_curve_id).unwrap();
        let result = edge_curve.validate_refs(&arena);
        assert!(result.is_err());
        assert!(
            matches!(result.err().unwrap(), ConversionStepItemError::SameSenseMismatch { keyword, same_sense } if keyword == "EDGE_CURVE" && same_sense)
        );
    }
}
