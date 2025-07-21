//! Representation of the STEP **manifold_solid_brep** entity (ISO 10303‑42:2003).
//!
//! ENTITY manifold_solid_brep
//!   SUBTYPE OF (solid_model);
//!   outer : closed_shell;
//! END_ENTITY;
//!
//! IP1: The dimensionality of a manifold_solid_brep shall be 3.
//! IP2: The extent of the manifold_solid_brep shall be finite and non-zero.
//! IP3: No vertex_point, undirected edge_curve (i.e, one which is not a oriented_edge), or undirected
//! face_surface (i.e., one which is not a oriented_face) referenced by a manifold_solid_brep shall intersect any other vertex_point, undirected edge_curve, or undirected face_surface referenced by the same
//! manifold_solid_brep.
//! IP4: Distinct loops referenced by the same face shall have no common vertexs.
//! NOTE This implies that distinct loops of the same face have no common edges. If geometry is present, distinct
//! loops of the same face do not intersect.
//! IP5: All topological elements of the manifold_solid_brep shall have defined associated geometry.
//! IP6: The shell normals shall agree with the B-rep normal and point away from the solid represented by
//! the B-rep.
//! IP7: Each face shall be referenced only once by the shells of the manifold_solid_brep.
//! IP8: Each oriented_edge in the manifold_solid_brep shall be referenced only once.
//! IP9: Each undirected edge shall be referenced exactly twice by the loops in the faces of the manifold_-
//! solid_brep’s shells.
//! IP10: The Euler equation shall be satisfied for the boundary representation, where the genus term shell_-
//! genus is the sum of the genus values for the shells of the B-rep.
//! IP11: A manifold_solid_brep, which is not a faceted_brep, shall not reference poly_loops.
//! IP12: A faceted_brep can reference only poly_loops as face boundaries.

use super::super::common::{
    check_keyword, expect_attr_len, expect_reference, expect_single_item, ConversionStepItemError,
    FromSimple, HasKeyword, StepItemCast,
};
use super::super::StepItem;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item::ValidateRefs;
use crate::step_item_map::{StepItemMap, StepItems};

#[derive(Debug, Clone)]
pub struct ManifoldSolidBrep {
    pub outer: EntityId,
}

impl HasKeyword for ManifoldSolidBrep {
    const KEYWORD: &'static str = "MANIFOLD_SOLID_BREP";
}

impl FromSimple for ManifoldSolidBrep {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have exactly 2 parameters (name, outer)
        expect_attr_len(&se, 2, Self::KEYWORD)?;

        // outer = #id
        let outer = expect_reference(&se.attrs[1], Self::KEYWORD)?;

        Ok(Self { outer })
    }
}

impl ValidateRefs for ManifoldSolidBrep {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        expect_single_item(arena, self.outer, "CLOSED_SHELL")?;
        Ok(())
    }
}

impl StepItemCast for ManifoldSolidBrep {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::ManifoldSolidBrep(msb) => Some(msb),
            _ => None,
        }
    }
}

impl From<ManifoldSolidBrep> for StepItem {
    fn from(msb: ManifoldSolidBrep) -> Self {
        StepItem::ManifoldSolidBrep(Box::new(msb))
    }
}

impl ManifoldSolidBrep {
    pub fn new(outer: EntityId) -> Self {
        Self { outer }
    }

    pub fn new_and_register(outer: EntityId, arena: &mut StepItemMap) -> EntityId {
        let msb = ManifoldSolidBrep::new(outer);
        arena.insert_default_id(StepItems::new_with_one_item(msb.into()))
    }
}

#[cfg(test)]
mod tests {
    use rk_calc::Vector3;

    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item::{CartesianPoint, ClosedShell};

    #[test]
    fn test_manifold_solid_brep_from_simple() {
        let se = SimpleEntity {
            keyword: "MANIFOLD_SOLID_BREP".to_string(),
            attrs: vec![
                Parameter::String("TestSolid".to_string()),
                Parameter::Reference(1),
            ],
        };

        let msb = ManifoldSolidBrep::from_simple(se).unwrap();
        assert_eq!(msb.outer, 1);
    }

    #[test]
    fn test_manifold_solid_brep_ifrom_simple_nvalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID_KEYWORD".to_string(),
            attrs: vec![
                Parameter::String("TestSolid".to_string()),
                Parameter::Reference(1),
            ],
        };

        let result = ManifoldSolidBrep::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::Unsupported { .. }
        ));
    }

    #[test]
    fn test_manifold_solid_brep_from_simple_invalid_attr_count() {
        let se = SimpleEntity {
            keyword: "MANIFOLD_SOLID_BREP".to_string(),
            attrs: vec![Parameter::String("TestSolid".to_string())], // Only one attribute
        };

        let result = ManifoldSolidBrep::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::AttrCount { expected, found, keyword } if expected == 2 && found == 1 && keyword == "MANIFOLD_SOLID_BREP"
        ));
    }

    #[test]
    fn test_manifold_solid_brep_from_simple_not_reference() {
        let se = SimpleEntity {
            keyword: "MANIFOLD_SOLID_BREP".to_string(),
            attrs: vec![
                Parameter::String("TestSolid".to_string()),
                Parameter::String("NotAReference".to_string()), // Invalid reference
            ],
        };

        let result = ManifoldSolidBrep::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NotReference { keyword } if keyword == "MANIFOLD_SOLID_BREP"
        ));
    }

    #[test]
    fn test_manifold_solid_brep_validate_refs() {
        let mut arena = StepItemMap::new();
        let outer_shell = ClosedShell::register_cube(
            1.0,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            &mut arena,
        )
        .unwrap();

        let msb = ManifoldSolidBrep::new(outer_shell);
        let result = msb.validate_refs(&arena);
        assert!(result.is_ok());
    }

    #[test]
    fn test_manifold_solid_brep_validate_refs_not_reference() {
        let arena = StepItemMap::new();
        let msb = ManifoldSolidBrep::new(999); // Non-existent reference

        let result = msb.validate_refs(&arena);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::UnresolvedRef { id } if id == 999
        ));
    }

    #[test]
    fn test_manifold_solid_brep_validate_refs_invalid_type() {
        let mut arena = StepItemMap::new();
        let point_id =
            arena.insert_default_id(StepItems::new_with_one_item(StepItem::CartesianPoint(
                CartesianPoint {
                    coords: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            )));

        let msb = ManifoldSolidBrep::new(point_id);

        let result = msb.validate_refs(&arena);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::TypeMismatch { expected, found, id } if expected == "CLOSED_SHELL" && found == "CARTESIAN_POINT" && id == point_id
        ));
    }
}
