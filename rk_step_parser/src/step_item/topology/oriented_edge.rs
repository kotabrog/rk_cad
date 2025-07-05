//! Representation of the STEP **oriented_edge** entity (ISO 10303‑42).
//!
//! ENTITY oriented_edge
//!   SUBTYPE OF (edge);
//!   edge_element : edge;
//!   orientation  : BOOLEAN;
//! DERIVE
//!   SELF\edge.edge_start : vertex := boolean_choose (SELF.orientation,
//!                         SELF.edge_element.edge_start,
//!                         SELF.edge_element.edge_end);
//!   SELF\edge.edge_end   : vertex := boolean_choose (SELF.orientation,
//!                         SELF.edge_element.edge_end,
//!                         SELF.edge_element.edge_start);
//! WHERE
//!   WR1: NOT ('TOPOLOGY_SCHEMA.ORIENTED_EDGE' IN TYPEOF (SELF.edge_element));
//! END_ENTITY;
//!
//! 注意：
//! - パラメータは5つあり、2,3つめはedge_start, edge_endを表すが、それはedge_elementから取得されるため、必ず「*」になっている必要がある

use super::super::common::{
    boolean_to_bool, check_keyword, expect_attr_len, expect_omitted, expect_reference,
    expect_single_item, ConversionStepItemError, FromSimple, HasKeyword, StepItemCast,
};
use super::super::StepItem;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item::ValidateRefs;
use crate::step_item_map::{StepItemMap, StepItems};

#[derive(Debug, Clone)]
pub struct OrientedEdge {
    pub edge_element: EntityId,
    pub orientation: bool,
}

impl HasKeyword for OrientedEdge {
    const KEYWORD: &'static str = "ORIENTED_EDGE";
}

impl FromSimple for OrientedEdge {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have 5 attributes (name, edge_start, edge_end, edge_element, orientation)
        expect_attr_len(&se, 5, Self::KEYWORD)?;

        // edge_start = VertexPoint::STAR;
        expect_omitted(&se.attrs[1], Self::KEYWORD)?;

        // edge_end = VertexPoint::STAR;
        expect_omitted(&se.attrs[2], Self::KEYWORD)?;

        // edge_element = #id
        let edge_element = expect_reference(&se.attrs[3], Self::KEYWORD)?;

        // orientation = true/false
        let orientation = boolean_to_bool(&se.attrs[4], Self::KEYWORD)?;

        Ok(Self {
            edge_element,
            orientation,
        })
    }
}

impl ValidateRefs for OrientedEdge {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        // edge_element must be a EdgeCurve
        expect_single_item(arena, self.edge_element, "EDGE_CURVE")?;

        Ok(())
    }
}

impl StepItemCast for OrientedEdge {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::OrientedEdge(oe) => Some(oe),
            _ => None,
        }
    }
}

impl From<OrientedEdge> for StepItem {
    fn from(oe: OrientedEdge) -> Self {
        StepItem::OrientedEdge(Box::new(oe))
    }
}

impl OrientedEdge {
    pub fn register_step_item_map(
        edge_element: EntityId,
        orientation: bool,
        arena: &mut StepItemMap,
    ) -> EntityId {
        let oriented_edge = OrientedEdge {
            edge_element,
            orientation,
        };
        arena.insert_default_id(StepItems::new_with_one_item(oriented_edge.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::super::EdgeCurve;
    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item::common::expect_single_item_cast;
    use rk_calc::Vector3;

    #[test]
    fn test_oriented_edge_from_simple() {
        let se = SimpleEntity {
            keyword: "ORIENTED_EDGE".into(),
            attrs: vec![
                Parameter::String("oriented_edge_1".to_string()),
                Parameter::Omitted,
                Parameter::Omitted,
                Parameter::Reference(2),
                Parameter::Logical(Some(true)),
            ],
        };

        let oriented_edge = OrientedEdge::from_simple(se).unwrap();
        assert_eq!(oriented_edge.edge_element, 2);
        assert!(oriented_edge.orientation);
    }

    #[test]
    fn test_oriented_edge_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID_EDGE".into(),
            attrs: vec![
                Parameter::String("invalid_edge_1".to_string()),
                Parameter::Omitted,
                Parameter::Omitted,
                Parameter::Reference(2),
                Parameter::Logical(Some(true)),
            ],
        };

        let result = OrientedEdge::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::Unsupported(_)
        ));
    }

    #[test]
    fn test_oriented_edge_from_simple_invalid_attr_len() {
        let se = SimpleEntity {
            keyword: "ORIENTED_EDGE".into(),
            attrs: vec![
                Parameter::String("oriented_edge_1".to_string()),
                Parameter::Omitted,
                Parameter::Reference(2),
                Parameter::Logical(Some(true)),
            ],
        };

        let result = OrientedEdge::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::AttrCount { expected, found, keyword } if keyword == "ORIENTED_EDGE" && expected == 5 && found == 4
        ));
    }

    #[test]
    fn test_oriented_edge_from_simple_not_reference() {
        let se = SimpleEntity {
            keyword: "ORIENTED_EDGE".into(),
            attrs: vec![
                Parameter::String("oriented_edge_1".to_string()),
                Parameter::Omitted,
                Parameter::Omitted,
                Parameter::String("not_a_reference".to_string()),
                Parameter::Logical(Some(true)),
            ],
        };

        let result = OrientedEdge::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NotReference { keyword, .. } if keyword == "ORIENTED_EDGE"
        ));
    }

    #[test]
    fn test_oriented_edge_validate_refs() {
        let mut arena = StepItemMap::new();
        let edge_curve_id = EdgeCurve::register_step_item_map(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
            Vector3::new(0.5, 0.5, 0.5),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
            true,
            &mut arena,
        );
        let edge_element = OrientedEdge::register_step_item_map(edge_curve_id, true, &mut arena);

        let oriented_edge = expect_single_item_cast::<OrientedEdge>(&arena, edge_element).unwrap();
        let result = oriented_edge.validate_refs(&arena);
        assert!(result.is_ok());
    }

    #[test]
    fn test_oriented_edge_validate_refs_invalid() {
        let mut arena = StepItemMap::new();
        let edge_element = OrientedEdge::register_step_item_map(999, true, &mut arena); // Invalid ID

        let oriented_edge = expect_single_item_cast::<OrientedEdge>(&arena, edge_element).unwrap();
        let result = oriented_edge.validate_refs(&arena);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::UnresolvedRef { id } if id == 999
        ));
    }
}
