//! Representation of the STEP **advanced_face** entity (AIM EXPRESS Library and ISO 10303‑42:2003).
//!
//! ENTITY advanced_face
//!    SUBTYPE OF (face_surface);
//!    WHERE
//!       WR1:
//!          SIZEOF([ 'STEP_MERGED_AP_SCHEMA.ELEMENTARY_SURFACE', 'STEP_MERGED_AP_SCHEMA.B_SPLINE_SURFACE', 'STEP_MERGED_AP_SCHEMA.SWEPT_SURFACE' ] * TYPEOF(face_geometry)) = 1;
//!       WR2:
//!          SIZEOF(QUERY (elp_fbnds <* QUERY (bnds <* bounds| ('STEP_MERGED_AP_SCHEMA.EDGE_LOOP' IN TYPEOF(bnds.bound)))| NOT (SIZEOF(QUERY (oe <* elp_fbnds.bound\path.edge_list| NOT ('STEP_MERGED_AP_SCHEMA.EDGE_CURVE' IN TYPEOF(oe\oriented_edge.edge_element)))) = 0))) = 0;
//!       WR3:
//!          SIZEOF(QUERY (elp_fbnds <* QUERY (bnds <* bounds| ('STEP_MERGED_AP_SCHEMA.EDGE_LOOP' IN TYPEOF(bnds.bound)))| NOT (SIZEOF(QUERY (oe <* elp_fbnds.bound\path.edge_list| NOT (SIZEOF([ 'STEP_MERGED_AP_SCHEMA.LINE', 'STEP_MERGED_AP_SCHEMA.CONIC', 'STEP_MERGED_AP_SCHEMA.POLYLINE', 'STEP_MERGED_AP_SCHEMA.SURFACE_CURVE', 'STEP_MERGED_AP_SCHEMA.B_SPLINE_CURVE' ] * TYPEOF(oe.edge_element\edge_curve.edge_geometry)) = 1))) = 0))) = 0;
//!       WR4:
//!          SIZEOF(QUERY (elp_fbnds <* QUERY (bnds <* bounds| ('STEP_MERGED_AP_SCHEMA.EDGE_LOOP' IN TYPEOF(bnds.bound)))| NOT (SIZEOF(QUERY (oe <* elp_fbnds.bound\path.edge_list| NOT ((('STEP_MERGED_AP_SCHEMA.VERTEX_POINT' IN TYPEOF(oe\edge.edge_start)) AND ('STEP_MERGED_AP_SCHEMA.CARTESIAN_POINT' IN TYPEOF(oe\edge.edge_start\vertex_point.vertex_geometry))) AND (('STEP_MERGED_AP_SCHEMA.VERTEX_POINT' IN TYPEOF(oe\edge.edge_end)) AND ('STEP_MERGED_AP_SCHEMA.CARTESIAN_POINT' IN TYPEOF(oe\edge.edge_end\vertex_point.vertex_geometry)))))) = 0))) = 0;
//!       WR5:
//!          SIZEOF(QUERY (elp_fbnds <* QUERY (bnds <* bounds| ('STEP_MERGED_AP_SCHEMA.EDGE_LOOP' IN TYPEOF(bnds.bound)))| ('STEP_MERGED_AP_SCHEMA.ORIENTED_PATH' IN TYPEOF(elp_fbnds.bound)))) = 0;
//!       WR6:
//!          NOT ('STEP_MERGED_AP_SCHEMA.SWEPT_SURFACE' IN TYPEOF(face_geometry)) OR (SIZEOF([ 'STEP_MERGED_AP_SCHEMA.LINE', 'STEP_MERGED_AP_SCHEMA.CONIC', 'STEP_MERGED_AP_SCHEMA.POLYLINE', 'STEP_MERGED_AP_SCHEMA.B_SPLINE_CURVE' ] * TYPEOF(face_geometry\swept_surface.swept_curve)) = 1);
//!       WR7:
//!          SIZEOF(QUERY (vlp_fbnds <* QUERY (bnds <* bounds| ('STEP_MERGED_AP_SCHEMA.VERTEX_LOOP' IN TYPEOF(bnds.bound)))| NOT (('STEP_MERGED_AP_SCHEMA.VERTEX_POINT' IN TYPEOF(vlp_fbnds\face_bound.bound\vertex_loop.loop_vertex)) AND ('STEP_MERGED_AP_SCHEMA.CARTESIAN_POINT' IN TYPEOF(vlp_fbnds\face_bound.bound\vertex_loop.loop_vertex\vertex_point.vertex_geometry))))) = 0;
//!       WR8:
//!          SIZEOF(QUERY (bnd <* bounds| NOT (SIZEOF([ 'STEP_MERGED_AP_SCHEMA.EDGE_LOOP', 'STEP_MERGED_AP_SCHEMA.VERTEX_LOOP' ] * TYPEOF(bnd.bound)) = 1))) = 0;
//!       WR9:
//!          SIZEOF(QUERY (elp_fbnds <* QUERY (bnds <* bounds| ('STEP_MERGED_AP_SCHEMA.EDGE_LOOP' IN TYPEOF(bnds.bound)))| NOT (SIZEOF(QUERY (oe <* elp_fbnds.bound\path.edge_list| ('STEP_MERGED_AP_SCHEMA.SURFACE_CURVE' IN TYPEOF(oe\oriented_edge.edge_element\edge_curve.edge_geometry)) AND NOT (SIZEOF(QUERY (sc_ag <* oe.edge_element\edge_curve.edge_geometry\surface_curve.associated_geometry| NOT ('STEP_MERGED_AP_SCHEMA.PCURVE' IN TYPEOF(sc_ag)))) = 0))) = 0))) = 0;
//!       WR10:
//!          (NOT ('STEP_MERGED_AP_SCHEMA.SWEPT_SURFACE' IN TYPEOF(face_geometry)) OR (NOT ('STEP_MERGED_AP_SCHEMA.POLYLINE' IN TYPEOF(face_geometry\swept_surface.swept_curve)) OR (SIZEOF(face_geometry\swept_surface.swept_curve\polyline.points) >= 3))) AND (SIZEOF(QUERY (elp_fbnds <* QUERY (bnds <* bounds| ('STEP_MERGED_AP_SCHEMA.EDGE_LOOP' IN TYPEOF(bnds.bound)))| NOT (SIZEOF(QUERY (oe <* elp_fbnds.bound\path.edge_list| ('STEP_MERGED_AP_SCHEMA.POLYLINE' IN TYPEOF(oe\oriented_edge.edge_element\edge_curve.edge_geometry)) AND NOT (SIZEOF(oe\oriented_edge.edge_element\edge_curve.edge_geometry\polyline.points) >= 3))) = 0))) = 0);
//! END_ENTITY;
//!
//! ENTITY face_surface
//!   SUBTYPE OF (face, geometric_representation_item);
//!   face_geometry : surface;
//!   same_sense    : BOOLEAN;
//! WHERE
//!   WR1: NOT ('GEOMETRY_SCHEMA.ORIENTED_SURFACE' IN TYPEOF (face_geometry));
//! END_ENTITY;
//!
//! IP1: The domain of the face_surface is formally defined to be the domain of its
//! face_geometry as trimmed by the loops, this domain does not include the bounding loops.
//! IP2: A face_surface has nonzero finite extent.
//! IP3: A face_surface is a manifold.
//! IP4: A face_surface is arcwise connected.
//! IP5: A face_surface has surface genus 0.
//! IP6: The loops are not part of the face domain.
//! IP7: Loop geometry shall be consistent with face geometry. This implies that any edge_curves or
//! vertex_points used in defining the loops bounding the face_surface shall lie on the face_geometry.
//! IP8: The loops of the face shall not intersect.
//!
//! ENTITY face
//!   SUPERTYPE OF (ONEOF (face_surface, subface, oriented_face))
//!   SUBTYPE OF (topological_representation_item);
//!   bounds : SET [1:?] OF face_bound;
//! WHERE
//!   WR1: NOT (mixed_loop_type_set (list_to_set (list_face_loops (SELF))));
//!   WR2: SIZEOF (QUERY (temp <* bounds |
//!         'TOPOLOGY_SCHEMA.FACE_OUTER_BOUND' IN TYPEOF (temp))) <= 1;
// END_ENTITY;
//!
//! ENTITY surface
//!   SUPERTYPE OF (ONEOF (
//!       elementary_surface,
//!       swept_surface,
//!       bounded_surface,
//!       offset_surface,
//!       surface_replica))
//!   SUBTYPE OF (geometric_representation_item);
//! END_ENTITY;
//!
//! 注意：
//! - `face_geometry` としては、現在は `PLANE` のみをサポートする。
//! - `face_bound` は一つだけの場合を現在はサポートする。
//! - `face_surface` の IP7: ループの幾何が曲面と整合していること、のチェックは現在は行っていない。IPチェックは別途行う予定

use super::super::common::{
    aggregate_to_reference, boolean_to_bool, check_keyword, expect_attr_len, expect_reference,
    expect_single_item, ConversionStepItemError, FromSimple, HasKeyword, StepItemCast,
};
use super::super::{Axis2Placement3D, FaceBound, Plane, StepItem};
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item::ValidateRefs;
use crate::step_item_map::{StepItemMap, StepItems};
use rk_calc::Vector3;

#[derive(Debug, Clone)]
pub struct AdvancedFace {
    pub face_bounds: Vec<EntityId>,
    pub face_geometry: EntityId,
    pub same_sense: bool,
}

impl HasKeyword for AdvancedFace {
    const KEYWORD: &'static str = "ADVANCED_FACE";
}

impl FromSimple for AdvancedFace {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // Must have exactly 3 parameters (name, face_bounds, face_geometry, same_sense).
        expect_attr_len(&se, 4, Self::KEYWORD)?;

        // face_bounds = [#id1, #id2, ...]
        let face_bounds = aggregate_to_reference(&se.attrs[1], Self::KEYWORD)?;

        // face_geometry = #id
        let face_geometry = expect_reference(&se.attrs[2], Self::KEYWORD)?;

        // same_sense = true/false
        let same_sense = boolean_to_bool(&se.attrs[3], Self::KEYWORD)?;

        Ok(Self {
            face_bounds,
            face_geometry,
            same_sense,
        })
    }
}

impl ValidateRefs for AdvancedFace {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        if self.face_bounds.is_empty() {
            return Err(ConversionStepItemError::NonEmptyList {
                keyword: Self::KEYWORD,
            });
        }

        // 現在は `face_bounds` は一つだけのケースをサポートする。
        if self.face_bounds.len() > 1 {
            return Err(ConversionStepItemError::SingleItemExpected {
                keyword: Self::KEYWORD,
            });
        }

        for bound in &self.face_bounds {
            expect_single_item(arena, *bound, "FACE_BOUND")?;
        }

        expect_single_item(arena, self.face_geometry, "PLANE")?;

        Ok(())
    }
}

impl StepItemCast for AdvancedFace {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::AdvancedFace(advanced_face) => Some(advanced_face),
            _ => None,
        }
    }
}

impl From<AdvancedFace> for StepItem {
    fn from(advanced_face: AdvancedFace) -> Self {
        StepItem::AdvancedFace(Box::new(advanced_face))
    }
}

impl AdvancedFace {
    pub fn new(face_bounds: Vec<EntityId>, face_geometry: EntityId, same_sense: bool) -> Self {
        Self {
            face_bounds,
            face_geometry,
            same_sense,
        }
    }

    pub fn new_and_register(
        face_bounds: Vec<EntityId>,
        face_geometry: EntityId,
        same_sense: bool,
        arena: &mut StepItemMap,
    ) -> EntityId {
        let advanced_face = Self::new(face_bounds, face_geometry, same_sense);
        arena.insert_default_id(StepItems::new_with_one_item(advanced_face.into()))
    }

    /// シンプルな正方形の平面を登録する。
    ///
    /// Attributes:
    /// - `size`: 中心から辺までの距離
    /// - `position`: 平面の中心位置
    /// - `normal`: 平面の法線ベクトル
    /// - `ref_direction`: 平面上の参照方向ベクトル
    /// - `arena`: 登録先の `StepItemMap`
    pub fn register_square(
        size: f64,
        position: Vector3,
        normal: Vector3,
        ref_direction: Vector3,
        arena: &mut StepItemMap,
    ) -> Result<EntityId, ConversionStepItemError> {
        let plane_id = Plane::register_step_item_map(position, normal, ref_direction, arena);
        let plane = arena
            .get_single_item(plane_id)
            .and_then(Plane::cast)
            .ok_or(ConversionStepItemError::UnresolvedRef { id: plane_id })?;
        let axis2_placement_3d = arena
            .get_single_item(plane.position)
            .and_then(Axis2Placement3D::cast)
            .ok_or(ConversionStepItemError::UnresolvedRef { id: plane.position })?;
        let [x, y, _z] = axis2_placement_3d.build_axes(arena)?;

        let point1 = position + x * size + y * size;
        let point2 = position + x * size - y * size;
        let point3 = position - x * size - y * size;
        let point4 = position - x * size + y * size;

        let face_bound = FaceBound::register_step_item_map_line_default_loop(
            vec![point1, point2, point3, point4],
            true,
            arena,
        );
        Ok(AdvancedFace::new_and_register(
            vec![face_bound],
            plane_id,
            true,
            arena,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_entity::Parameter;

    #[test]
    fn test_advanced_face_from_simple() {
        let se = SimpleEntity {
            keyword: "ADVANCED_FACE".to_string(),
            attrs: vec![
                Parameter::String("AdvancedFace1".to_string()),
                Parameter::Aggregate(vec![
                    Parameter::Reference(1), // face_bound_1
                    Parameter::Reference(2), // face_bound_2
                ]),
                Parameter::Reference(3),        // face_geometry
                Parameter::Logical(Some(true)), // same_sense
            ],
        };
        let advanced_face = AdvancedFace::from_simple(se).unwrap();
        assert_eq!(advanced_face.face_bounds.len(), 2);
        assert_eq!(advanced_face.face_geometry, 3);
        assert!(advanced_face.same_sense);
    }

    #[test]
    fn test_advanced_face_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID".to_string(),
            attrs: vec![
                Parameter::String("AdvancedFace1".to_string()),
                Parameter::Aggregate(vec![Parameter::Reference(1)]),
                Parameter::Reference(2),
                Parameter::Logical(Some(true)),
            ],
        };
        let result = AdvancedFace::from_simple(se);
        assert!(result.is_err());
    }

    #[test]
    fn test_advanced_face_from_simple_invalid_attr_len() {
        let se = SimpleEntity {
            keyword: "ADVANCED_FACE".to_string(),
            attrs: vec![
                Parameter::String("AdvancedFace1".to_string()),
                Parameter::Aggregate(vec![Parameter::Reference(1)]),
                Parameter::Reference(2),
                // Missing same_sense
            ],
        };
        let result = AdvancedFace::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::AttrCount { expected, found, keyword } if expected == 4 && found == 3 && keyword == "ADVANCED_FACE"
        ));
    }

    #[test]
    fn test_advanced_face_from_simple_not_aggregate() {
        let se = SimpleEntity {
            keyword: "ADVANCED_FACE".to_string(),
            attrs: vec![
                Parameter::String("AdvancedFace1".to_string()),
                Parameter::Reference(1), // Not an aggregate
                Parameter::Reference(2),
                Parameter::Logical(Some(true)),
            ],
        };
        let result = AdvancedFace::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NotAggregate { keyword } if keyword == "ADVANCED_FACE"
        ));
    }

    #[test]
    fn test_advanced_face_from_simple_not_reference_aggregate() {
        let se = SimpleEntity {
            keyword: "ADVANCED_FACE".to_string(),
            attrs: vec![
                Parameter::String("AdvancedFace1".to_string()),
                Parameter::Aggregate(vec![Parameter::String("NotAReference".to_string())]), // Not a reference
                Parameter::Reference(2),
                Parameter::Logical(Some(true)),
            ],
        };
        let result = AdvancedFace::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NotReference { keyword } if keyword == "ADVANCED_FACE"
        ));
    }

    #[test]
    fn test_advanced_face_from_simple_not_reference() {
        let se = SimpleEntity {
            keyword: "ADVANCED_FACE".to_string(),
            attrs: vec![
                Parameter::String("AdvancedFace1".to_string()),
                Parameter::Aggregate(vec![Parameter::Reference(1)]),
                Parameter::String("NotAReference".to_string()), // Not a reference
                Parameter::Logical(Some(true)),
            ],
        };
        let result = AdvancedFace::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NotReference { keyword } if keyword == "ADVANCED_FACE"
        ));
    }

    #[test]
    fn test_advanced_face_from_simple_not_boolean() {
        let se = SimpleEntity {
            keyword: "ADVANCED_FACE".to_string(),
            attrs: vec![
                Parameter::String("AdvancedFace1".to_string()),
                Parameter::Aggregate(vec![Parameter::Reference(1)]),
                Parameter::Reference(2),
                Parameter::String("NotABoolean".to_string()), // Not a boolean
            ],
        };
        let result = AdvancedFace::from_simple(se);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NotBoolean { keyword } if keyword == "ADVANCED_FACE"
        ));
    }

    #[test]
    fn test_advanced_face_validate_refs() {
        let mut arena = StepItemMap::new();
        let face_bound = FaceBound::register_step_item_map_line_default_loop(
            vec![
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            ],
            true,
            &mut arena,
        );
        let plane = Plane::register_step_item_map(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
            &mut arena,
        );
        let advanced_face = AdvancedFace::new(vec![face_bound], plane, true);
        assert!(advanced_face.validate_refs(&arena).is_ok());
    }

    #[test]
    fn test_advanced_face_validate_refs_invalid() {
        let mut arena = StepItemMap::new();
        let face_bound = FaceBound::register_step_item_map_line_default_loop(
            vec![
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(1.0, 1.0, 0.0),
            ],
            true,
            &mut arena,
        );
        let advanced_face = AdvancedFace::new(
            vec![face_bound],
            999, // Invalid reference
            true,
        );
        let result = advanced_face.validate_refs(&arena);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::UnresolvedRef { id } if id == 999
        ));
    }

    #[test]
    fn test_advanced_face_validate_refs_empty_bounds() {
        let arena = StepItemMap::new();
        let advanced_face = AdvancedFace::new(vec![], 1, true);
        let result = advanced_face.validate_refs(&arena);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::NonEmptyList { keyword } if keyword == "ADVANCED_FACE"
        ));
    }

    #[test]
    fn test_advanced_face_validate_refs_wrong_type() {
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(Plane { position: 2 }.into()),
        );
        let advanced_face = AdvancedFace::new(vec![1], 1, true);
        let result = advanced_face.validate_refs(&arena);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ConversionStepItemError::TypeMismatch { expected, found, id } if expected == "FACE_BOUND" && found == "PLANE" && id == 1
        ));
    }
}
