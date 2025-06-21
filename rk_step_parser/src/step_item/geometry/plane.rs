//! -----------------------------------------------------------------------------
//! ISO 10303-42 ― ENTITY `PLANE` 仕様要約
//!
//! ENTITY plane
//!   SUBTYPE OF (elementary_surface);
//! END_ENTITY;
//!
//! ENTITY elementary_surface
//!   SUPERTYPE OF (ONEOF(plane, cylindrical_surface, conical_surface,
//!                       spherical_surface, toroidal_surface))
//!   SUBTYPE OF (surface);
//!   position : axis2_placement_3d;
//! END_ENTITY;
//!
//! * 現在はelementary_surfaceは作成せず、直接planeにaxis2_placement_3dを持たせる。
//! -----------------------------------------------------------------------------

use super::super::common::{
    check_keyword, expect_attr_len, expect_reference, expect_single_item, ConversionStepItemError,
    FromSimple, HasKeyword, StepItemCast,
};
use super::super::StepItem;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item::ValidateRefs;
use crate::step_item_map::StepItemMap;

#[derive(Debug, Clone)]
pub struct Plane {
    pub position: EntityId, // Axis2Placement3D
}

impl HasKeyword for Plane {
    const KEYWORD: &'static str = "PLANE";
}

impl FromSimple for Plane {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have exactly 2 parameters (name, position).
        expect_attr_len(&se, 2, Self::KEYWORD)?;

        // position = #id
        let position = expect_reference(&se.attrs[1], Self::KEYWORD)?;

        Ok(Plane { position })
    }
}

impl ValidateRefs for Plane {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        // position must be an AXIS2_PLACEMENT_3D
        expect_single_item(arena, self.position, "AXIS2_PLACEMENT_3D")?;
        Ok(())
    }
}

impl StepItemCast for Plane {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::Plane(plane) => Some(plane),
            _ => None,
        }
    }
}

impl From<Plane> for StepItem {
    fn from(plane: Plane) -> Self {
        StepItem::Plane(Box::new(plane))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item::{Axis2Placement3D, CartesianPoint};
    use crate::step_item_map::StepItems;
    use rk_calc::Vector3;

    #[test]
    fn test_plane_from_simple() {
        let se = SimpleEntity {
            keyword: "PLANE".to_string(),
            attrs: vec![Parameter::String("".to_string()), Parameter::Reference(1)],
        };

        let plane = Plane::from_simple(se).unwrap();
        assert_eq!(plane.position, 1);
    }

    #[test]
    fn test_plane_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID".to_string(),
            attrs: vec![Parameter::String("".to_string()), Parameter::Reference(1)],
        };

        let result = Plane::from_simple(se).unwrap_err();
        assert!(matches!(result, ConversionStepItemError::Unsupported(_)));
    }

    #[test]
    fn test_plane_from_simple_invalid_attr_len() {
        let se = SimpleEntity {
            keyword: "PLANE".to_string(),
            attrs: vec![Parameter::String("".to_string())], // Missing position
        };

        let result = Plane::from_simple(se).unwrap_err();
        assert!(
            matches!(result, ConversionStepItemError::AttrCount { expected, found, keyword } if expected == 2 && found == 1 && keyword == "PLANE")
        );
    }

    #[test]
    fn test_plane_from_simple_invalid_reference() {
        let se = SimpleEntity {
            keyword: "PLANE".to_string(),
            attrs: vec![Parameter::String("".to_string()), Parameter::Real(1.0)],
        };

        let result = Plane::from_simple(se).unwrap_err();
        assert!(
            matches!(result, ConversionStepItemError::NotReference { keyword } if keyword == "PLANE")
        );
    }

    #[test]
    fn test_plane_validate_refs() {
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(
                Axis2Placement3D {
                    location: 2,
                    axis: None,
                    ref_direction: None,
                }
                .into(),
            ),
        );

        let plane = Plane { position: 1 };
        assert!(plane.validate_refs(&arena).is_ok());
    }

    #[test]
    fn test_plane_validate_refs_invalid_position() {
        let arena = StepItemMap::new();

        let plane = Plane { position: 1 };
        let result = plane.validate_refs(&arena);
        assert!(matches!(result, Err(ConversionStepItemError::UnresolvedRef { id }) if id == 1));
    }

    #[test]
    fn test_plane_validate_refs_wrong_type() {
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(
                CartesianPoint {
                    coords: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            ),
        );

        let plane = Plane { position: 1 };
        let result = plane.validate_refs(&arena);
        assert!(
            matches!(result, Err(ConversionStepItemError::TypeMismatch { expected, found, id }) if expected == "AXIS2_PLACEMENT_3D" && found == "CARTESIAN_POINT" && id == 1)
        );
    }
}
