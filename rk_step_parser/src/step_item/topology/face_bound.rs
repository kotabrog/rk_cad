//! Representation of the STEP **face_bound** entity (ISO 10303‑42).
//!
//! ENTITY face_bound
//!   SUBTYPE OF (topological_representation_item);
//!   bound       : loop;
//!   orientation : BOOLEAN;
//! END_ENTITY;
//!
//! 注意：
//! - bound は loop を受け取るが、現在は `EdgeLoop` のみをサポートする。

use super::super::common::{
    boolean_to_bool, check_keyword, expect_attr_len, expect_reference, expect_single_item,
    ConversionStepItemError, FromSimple, HasKeyword, StepItemCast,
};
use super::super::StepItem;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item::ValidateRefs;
use crate::step_item_map::{StepItemMap, StepItems};

#[derive(Debug, Clone)]
pub struct FaceBound {
    pub bound: EntityId,
    pub orientation: bool,
}

impl HasKeyword for FaceBound {
    const KEYWORD: &'static str = "FACE_BOUND";
}

impl FromSimple for FaceBound {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have exactly 3 parameters (name, bound, orientation).
        expect_attr_len(&se, 3, Self::KEYWORD)?;

        // bound = #id
        let bound = expect_reference(&se.attrs[1], Self::KEYWORD)?;

        // orientation = true/false
        let orientation = boolean_to_bool(&se.attrs[2], Self::KEYWORD)?;

        Ok(Self { bound, orientation })
    }
}

impl ValidateRefs for FaceBound {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        expect_single_item(arena, self.bound, "EDGE_LOOP")?;

        Ok(())
    }
}

impl StepItemCast for FaceBound {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::FaceBound(face_bound) => Some(face_bound),
            _ => None,
        }
    }
}

impl From<FaceBound> for StepItem {
    fn from(face_bound: FaceBound) -> Self {
        StepItem::FaceBound(Box::new(face_bound))
    }
}

impl FaceBound {
    pub fn new(bound: EntityId, orientation: bool) -> Self {
        Self { bound, orientation }
    }

    pub fn new_and_register(
        bound: EntityId,
        orientation: bool,
        arena: &mut StepItemMap,
    ) -> EntityId {
        let face_bound = Self::new(bound, orientation);
        arena.insert_default_id(StepItems::new_with_one_item(face_bound.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item::EdgeLoop;
    use rk_calc::Vector3;

    #[test]
    fn test_face_bound_from_simple() {
        let se = SimpleEntity {
            keyword: "FACE_BOUND".to_string(),
            attrs: vec![
                Parameter::String("FaceBound1".to_string()),
                Parameter::Reference(1),
                Parameter::Logical(Some(true)),
            ],
        };
        let face_bound = FaceBound::from_simple(se).unwrap();
        assert_eq!(face_bound.bound, 1);
        assert!(face_bound.orientation);
    }

    #[test]
    fn test_face_bound_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID".to_string(),
            attrs: vec![
                Parameter::String("FaceBound1".to_string()),
                Parameter::Reference(1),
                Parameter::Logical(Some(true)),
            ],
        };
        let result = FaceBound::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::Unsupported(_)
        ));
    }

    #[test]
    fn test_face_bound_from_simple_invalid_attr_len() {
        let se = SimpleEntity {
            keyword: "FACE_BOUND".to_string(),
            attrs: vec![
                Parameter::String("FaceBound1".to_string()),
                Parameter::Reference(1),
            ], // Missing orientation
        };
        let result = FaceBound::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::AttrCount { expected, found, keyword } if expected == 3 && found == 2 && keyword == "FACE_BOUND"
        ));
    }

    #[test]
    fn test_face_bound_from_simple_not_reference() {
        let se = SimpleEntity {
            keyword: "FACE_BOUND".to_string(),
            attrs: vec![
                Parameter::String("FaceBound1".to_string()),
                Parameter::String("NotAReference".to_string()),
                Parameter::Logical(Some(true)),
            ],
        };
        let result = FaceBound::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NotReference { keyword } if keyword == "FACE_BOUND"
        ));
    }

    #[test]
    fn test_face_bound_validate_refs() {
        let mut arena = StepItemMap::new();
        let points = vec![
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];
        let edge_loop_id = EdgeLoop::register_step_item_map_line_default_loop(points, &mut arena);
        let face_bound = FaceBound::new(edge_loop_id, true);
        let result = face_bound.validate_refs(&arena);
        assert!(result.is_ok());
    }

    #[test]
    fn test_face_bound_validate_refs_invalid() {
        let arena = StepItemMap::new();
        let face_bound = FaceBound::new(999, true); // Invalid ID not in arena
        let result = face_bound.validate_refs(&arena);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::UnresolvedRef { id } if id == 999
        ));
    }
}
