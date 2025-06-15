//! Representation of the STEP **vertex_point** entity (ISO 10303‑42).
//!
//! ENTITY vertex_point
//!   SUBTYPE OF (vertex, geometric_representation_item);
//!   vertex_geometry : point;
//! END_ENTITY;
//!
//! 注意：
//! - `vertex_geometry` は、本来は `point` 型の参照であるため、
//!   point エンティティまたはそのすべての下位型を取れるが、
//!   現在は`CARTESIAN_POINT` のみを受け入れる。

use super::super::common::{
    check_keyword, expect_attr_len, expect_reference, expect_single_item, expect_single_item_cast,
    ConversionStepItemError, FromSimple, HasKeyword, StepItemCast, ValidateRefs,
};
use super::super::geometry::CartesianPoint;
use super::super::StepItem;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item_map::StepItemMap;
use rk_calc::Vector3;

/// Represents a STEP vertex_point entity.
#[derive(Debug, Clone, PartialEq)]
pub struct VertexPoint {
    pub vertex_geometry: EntityId,
}

impl HasKeyword for VertexPoint {
    const KEYWORD: &'static str = "VERTEX_POINT";
}

impl FromSimple for VertexPoint {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Expect 2 attributes: name and vertex_geometry
        expect_attr_len(&se, 2, Self::KEYWORD)?;

        // vertex_geometry is a reference to a point (CARTESIAN_POINT)
        let vertex_geometry = expect_reference(&se.attrs[1], Self::KEYWORD)?;

        Ok(Self { vertex_geometry })
    }
}

impl ValidateRefs for VertexPoint {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        // vertex_geometry must be a CARTESIAN_POINT
        expect_single_item(arena, self.vertex_geometry, "CARTESIAN_POINT")?;
        Ok(())
    }
}

impl StepItemCast for VertexPoint {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::VertexPoint(vp) => Some(vp),
            _ => None,
        }
    }
}

impl From<VertexPoint> for StepItem {
    fn from(vp: VertexPoint) -> Self {
        StepItem::VertexPoint(Box::new(vp))
    }
}

impl VertexPoint {
    /// vertex_geometry の値を取得する
    pub fn vertex_geometry_value(
        &self,
        arena: &StepItemMap,
    ) -> Result<Vector3, ConversionStepItemError> {
        // vertex_geometry は CARTESIAN_POINT の参照であることを確認
        let point = expect_single_item_cast::<CartesianPoint>(arena, self.vertex_geometry)?;
        Ok(point.coords)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::geometry::Direction;
    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item_map::StepItems;

    #[test]
    fn test_vertex_point_from_simple() {
        let se = SimpleEntity {
            keyword: "VERTEX_POINT".into(),
            attrs: vec![Parameter::String("''".into()), Parameter::Reference(1)],
        };

        let vp = VertexPoint::from_simple(se).unwrap();
        assert_eq!(vp.vertex_geometry, 1);
    }

    #[test]
    fn test_vertex_point_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID".into(),
            attrs: vec![Parameter::String("''".into()), Parameter::Reference(1)],
        };

        let err = VertexPoint::from_simple(se).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::Unsupported(_)));
    }

    #[test]
    fn test_vertex_point_from_simple_not_reference() {
        let se = SimpleEntity {
            keyword: "VERTEX_POINT".into(),
            attrs: vec![Parameter::String("''".into()), Parameter::Real(1.0)],
        };

        let err = VertexPoint::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::NotReference { keyword } if keyword == "VERTEX_POINT")
        );
    }

    #[test]
    fn test_vertex_point_from_simple_attr_count() {
        let se = SimpleEntity {
            keyword: "VERTEX_POINT".into(),
            attrs: vec![Parameter::String("''".into())],
        };

        let err = VertexPoint::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::AttrCount { expected, found, keyword } if expected == 2 && found == 1 && keyword == "VERTEX_POINT")
        );
    }

    #[test]
    fn test_vertex_point_validate_refs() {
        let mut arena = StepItemMap::new();
        let vp = VertexPoint { vertex_geometry: 1 };
        arena.insert(
            1,
            StepItems::new_with_one_item(
                CartesianPoint {
                    coords: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            ),
        );

        assert!(vp.validate_refs(&arena).is_ok());
    }

    #[test]
    fn test_vertex_point_validate_refs_invalid() {
        let mut arena = StepItemMap::new();
        let vp = VertexPoint { vertex_geometry: 1 };
        arena.insert(
            1,
            StepItems::new_with_one_item(
                // This is not a CartesianPoint, so it should fail validation
                Direction {
                    vec: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            ),
        );

        let err = vp.validate_refs(&arena).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::TypeMismatch { expected, found, id }
            if expected == "CARTESIAN_POINT" && found == "DIRECTION" && id == 1)
        );
    }

    #[test]
    fn test_vertex_point_validate_refs_unresolved() {
        let arena: StepItemMap = StepItemMap::new();
        let vp = VertexPoint {
            vertex_geometry: 999, // Unresolved reference
        };

        let err = vp.validate_refs(&arena).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::UnresolvedRef { id } if id == 999));
    }
}
