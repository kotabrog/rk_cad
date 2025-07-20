//! Representation of the STEP **closed_shell** entity (ISO 10303‑42:2003).
//!
//! ENTITY closed_shell
//!   SUBTYPE OF (connected_face_set);
//! END_ENTITY;
//!
//! IP1: Every edge shall be referenced exactly twice by the face_bounds of the faces.
//! IP2: Each oriented_edge reference shall be unique.
//! IP3: No edge shall be referenced by more than two faces.
//! IP4: Distinct faces of the shell do not intersect, but may share edges, or vertices.
//! IP5: Distinct edges do not intersect, but may share vertices.
//! IP6: Each face reference shall be unique.
//! IP7: The loops of the shell shall not be a mixture of poly_loops and other loop types.
//! IP8: The closed_shell shall be an oriented arcwise connected-manifold.
//! IP9: The Euler equation shall be satisfied.
//! IP10: The topological normal to each face of the closed_shell shall be consistent with the topological
//! normal to the closed_shell. This implies that the topological normal to each face, after taking account
//! of orientation, if present, shall point from the finite region bounded by the closed_shell into the infinite
//! region outside.
//!
//! ENTITY connected_face_set
//!   SUPERTYPE OF (ONEOF (closed_shell, open_shell))
//!   SUBTYPE OF (topological_representation_item);
//!   cfs_faces : SET [1:?] OF face;
//! END_ENTITY;
//!
//! IP1: The union of the domains of the faces and their bounding loops shall be arcwise connected.
//!
//! 注意：
//! - 現在はIPチェックを一切行っていない
//! - `cfs_faces` は `advanced_face` のみを参照することとしている

use super::super::common::{
    aggregate_to_reference, check_keyword, expect_attr_len, expect_single_item,
    ConversionStepItemError, FromSimple, HasKeyword, StepItemCast,
};
use super::super::StepItem;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item::ValidateRefs;
use crate::step_item_map::{StepItemMap, StepItems};

#[derive(Debug, Clone)]
pub struct ClosedShell {
    pub cfs_faces: Vec<EntityId>,
}

impl HasKeyword for ClosedShell {
    const KEYWORD: &'static str = "CLOSED_SHELL";
}

impl FromSimple for ClosedShell {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have exactly 2 parameters (name, cfs_faces)
        expect_attr_len(&se, 2, Self::KEYWORD)?;

        // cfs_faces = [#id1, #id2, ...]
        let cfs_faces = aggregate_to_reference(&se.attrs[1], Self::KEYWORD)?;

        Ok(Self { cfs_faces })
    }
}

impl ValidateRefs for ClosedShell {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        if self.cfs_faces.is_empty() {
            return Err(ConversionStepItemError::NonEmptyList {
                keyword: Self::KEYWORD,
            });
        }

        for face_id in &self.cfs_faces {
            expect_single_item(arena, *face_id, "ADVANCED_FACE")?;
        }

        Ok(())
    }
}

impl StepItemCast for ClosedShell {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::ClosedShell(shell) => Some(shell),
            _ => None,
        }
    }
}

impl From<ClosedShell> for StepItem {
    fn from(shell: ClosedShell) -> Self {
        StepItem::ClosedShell(Box::new(shell))
    }
}

impl ClosedShell {
    pub fn new(cfs_faces: Vec<EntityId>) -> Self {
        Self { cfs_faces }
    }

    pub fn new_and_register(cfs_faces: Vec<EntityId>, arena: &mut StepItemMap) -> EntityId {
        let shell = ClosedShell::new(cfs_faces);
        arena.insert_default_id(StepItems::new_with_one_item(shell.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item::{AdvancedFace, CartesianPoint};
    use rk_calc::Vector3;

    #[test]
    fn test_closed_shell_from_simple() {
        let se = SimpleEntity {
            keyword: "CLOSED_SHELL".to_string(),
            attrs: vec![
                Parameter::String("ClosedShell1".to_string()),
                Parameter::Aggregate(vec![Parameter::Reference(1), Parameter::Reference(2)]),
            ],
        };

        let closed_shell = ClosedShell::from_simple(se).unwrap();
        assert_eq!(closed_shell.cfs_faces, vec![1, 2]);
    }

    #[test]
    fn test_closed_shell_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "OPEN_SHELL".to_string(),
            attrs: vec![
                Parameter::String("OpenShell1".to_string()),
                Parameter::Aggregate(vec![Parameter::Reference(1), Parameter::Reference(2)]),
            ],
        };

        let result = ClosedShell::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::Unsupported(_),
        ));
    }

    #[test]
    fn test_closed_shell_from_simple_invalid_attr_len() {
        let se = SimpleEntity {
            keyword: "CLOSED_SHELL".to_string(),
            attrs: vec![Parameter::String("ClosedShell1".to_string())], // Only one attribute
        };
        let result = ClosedShell::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::AttrCount { expected, found, keyword } if expected == 2 && found == 1 && keyword == "CLOSED_SHELL"
        ));
    }

    #[test]
    fn test_closed_shell_from_simple_not_aggregate() {
        let se = SimpleEntity {
            keyword: "CLOSED_SHELL".to_string(),
            attrs: vec![
                Parameter::String("ClosedShell1".to_string()),
                Parameter::String("NotAnAggregate".to_string()), // Invalid aggregate
            ],
        };
        let result = ClosedShell::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NotAggregate { keyword } if keyword == "CLOSED_SHELL"
        ));
    }

    #[test]
    fn test_closed_shell_from_simple_not_reference() {
        let se = SimpleEntity {
            keyword: "CLOSED_SHELL".to_string(),
            attrs: vec![
                Parameter::String("ClosedShell1".to_string()),
                Parameter::Aggregate(vec![Parameter::String("NotAReference".to_string())]), // Invalid reference
            ],
        };
        let result = ClosedShell::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NotReference { keyword } if keyword == "CLOSED_SHELL"
        ));
    }

    #[test]
    fn test_closed_shell_validate_refs() {
        let mut arena = StepItemMap::new();
        // 正方形の各面を作成
        let square1 = AdvancedFace::register_square(
            0.5,
            Vector3::new(0.0, 0.0, 0.5),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
            &mut arena,
        )
        .unwrap();
        let square2 = AdvancedFace::register_square(
            0.5,
            Vector3::new(0.0, 0.0, -0.5),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(-1.0, 0.0, 0.0),
            &mut arena,
        )
        .unwrap();
        let square3 = AdvancedFace::register_square(
            0.5,
            Vector3::new(0.5, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            &mut arena,
        )
        .unwrap();
        let square4 = AdvancedFace::register_square(
            0.5,
            Vector3::new(-0.5, 0.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            &mut arena,
        )
        .unwrap();
        let square5 = AdvancedFace::register_square(
            0.5,
            Vector3::new(0.0, 0.5, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            &mut arena,
        )
        .unwrap();
        let square6 = AdvancedFace::register_square(
            0.5,
            Vector3::new(0.0, -0.5, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 0.0, -1.0),
            &mut arena,
        )
        .unwrap();

        let closed_shell =
            ClosedShell::new(vec![square1, square2, square3, square4, square5, square6]);
        assert!(closed_shell.validate_refs(&arena).is_ok());
    }

    #[test]
    fn test_closed_shell_validate_refs_invalid() {
        let arena = StepItemMap::new();
        let closed_shell = ClosedShell::new(vec![1, 2, 3]); // 1, 2, 3 は存在しないID

        let result = closed_shell.validate_refs(&arena);
        assert!(matches!(
            result,
            Err(ConversionStepItemError::UnresolvedRef { id }) if id == 1
        ));
    }

    #[test]
    fn test_closed_shell_validate_refs_empty() {
        let arena = StepItemMap::new();
        let closed_shell = ClosedShell::new(vec![]);
        let result = closed_shell.validate_refs(&arena);
        assert!(matches!(
            result,
            Err(ConversionStepItemError::NonEmptyList { keyword }) if keyword == "CLOSED_SHELL"
        ));
    }

    #[test]
    fn test_closed_shell_validate_refs_invalid_face_type() {
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
        let closed_shell = ClosedShell::new(vec![1]); // 1 は CartesianPoint
        let result = closed_shell.validate_refs(&arena);
        assert!(matches!(
            result,
            Err(ConversionStepItemError::TypeMismatch { expected, found, id } ) if expected == "ADVANCED_FACE" && found == "CARTESIAN_POINT" && id == 1
        ));
    }
}
