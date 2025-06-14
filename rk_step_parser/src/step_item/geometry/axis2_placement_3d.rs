//! Representation of the STEP **axis2_placement_3d** entity (ISO 10303‑42).
//!
//! ENTITY axis2_placement_3d
//!   SUBTYPE OF (placement);
//!     axis         : OPTIONAL direction;
//!     ref_direction: OPTIONAL direction;
//! DERIVE
//!     p : LIST [3:3] OF direction := build_axes(axis, ref_direction);
//! WHERE
//!     WR1: SELF\placement.location.dim = 3;
//!     WR2: NOT EXISTS(axis)         OR (axis.dim = 3);
//!     WR3: NOT EXISTS(ref_direction) OR (ref_direction.dim = 3);
//! WR4: (NOT EXISTS(axis)) OR (NOT EXISTS(ref_direction)) OR
//!     (cross_product(axis, ref_direction).magnitude > 0.0);
//! END_ENTITY;

use super::super::common::{
    check_keyword, expect_attr_len, expect_reference, expect_reference_or_null, expect_single_item,
    expect_single_item_cast, ConversionStepItemError, FromSimple, HasKeyword, StepItemCast,
    ValidateRefs,
};
use super::super::StepItem;
use super::Direction;
use crate::step_entity::{EntityId, SimpleEntity};
use crate::step_item_map::StepItemMap;
use rk_calc::Vector3;

/// 解析直後（参照未解決）の Axis2Placement3D
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axis2Placement3D {
    pub location: EntityId,              // CartesianPoint
    pub axis: Option<EntityId>,          // Direction
    pub ref_direction: Option<EntityId>, // Direction
}

impl HasKeyword for Axis2Placement3D {
    const KEYWORD: &'static str = "AXIS2_PLACEMENT_3D";
}

impl FromSimple for Axis2Placement3D {
    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // name, location, axis, ref_direction
        expect_attr_len(&se, 4, Self::KEYWORD)?;

        // location = #id
        let location = expect_reference(&se.attrs[1], Self::KEYWORD)?;

        // axis = #id or $
        let axis = expect_reference_or_null(&se.attrs[2], Self::KEYWORD)?;

        // ref_direction = #id or $
        let ref_direction = expect_reference_or_null(&se.attrs[3], Self::KEYWORD)?;

        Ok(Self {
            location,
            axis,
            ref_direction,
        })
    }
}

impl ValidateRefs for Axis2Placement3D {
    fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        // location は必須
        expect_single_item(arena, self.location, "CARTESIAN_POINT")?;

        let axis = self.axis_value(arena)?;
        let ref_direction = self.ref_direction_value(arena)?;

        // 平行性チェック
        // 仕様では axis と ref_direction がデフォルト値の場合は平行でないとされているが、
        // 現状ではデフォルト値を使用する場合も平行性チェックを行っている
        if axis.dot(&ref_direction).abs() > 1.0 - 1.0e-6 {
            return Err(ConversionStepItemError::AxisRefDirectionNotOrthogonal);
        }

        Ok(())
    }
}

impl StepItemCast for Axis2Placement3D {
    fn cast(item: &StepItem) -> Option<&Self> {
        match item {
            StepItem::Axis2Placement3D(boxed) => Some(boxed),
            _ => None,
        }
    }
}

impl From<Axis2Placement3D> for StepItem {
    fn from(ap: Axis2Placement3D) -> Self {
        StepItem::Axis2Placement3D(Box::new(ap))
    }
}

impl Axis2Placement3D {
    /// axis の値を取得する
    pub fn axis_value(&self, arena: &StepItemMap) -> Result<Vector3, ConversionStepItemError> {
        if let Some(axis_id) = self.axis {
            let axis_item = expect_single_item_cast::<Direction>(arena, axis_id)?;
            Ok(axis_item.normalize())
        } else {
            Ok(Vector3::new(0.0, 0.0, 1.0))
        }
    }

    /// ref_direction の値を取得する
    pub fn ref_direction_value(
        &self,
        arena: &StepItemMap,
    ) -> Result<Vector3, ConversionStepItemError> {
        if let Some(ref_dir_id) = self.ref_direction {
            let ref_dir_item = expect_single_item_cast::<Direction>(arena, ref_dir_id)?;
            Ok(ref_dir_item.normalize())
        } else {
            Ok(Vector3::new(1.0, 0.0, 0.0))
        }
    }

    /// x 軸の値を計算する
    fn calc_x_value(&self, z: Vector3, a: Vector3) -> Result<Vector3, ConversionStepItemError> {
        let x_raw = a - z * z.dot(&a);
        x_raw
            .normalize_checked()
            .map_err(|_| ConversionStepItemError::NormalizeFailed {
                keyword: Self::KEYWORD,
            })
    }

    /// x 軸の値を取得する
    pub fn x_value(&self, arena: &StepItemMap) -> Result<Vector3, ConversionStepItemError> {
        let z = self.axis_value(arena)?;
        let a = self.ref_direction_value(arena)?;
        self.calc_x_value(z, a)
    }

    /// 3 つの軸を計算して返す
    pub fn build_axes(&self, arena: &StepItemMap) -> Result<[Vector3; 3], ConversionStepItemError> {
        let z = self.axis_value(arena)?;
        let a = self.ref_direction_value(arena)?;

        // x 軸の計算
        let x = self.calc_x_value(z, a)?;

        // y 軸は z と x の外積
        let y = z.cross(&x);

        Ok([x, y, z])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_entity::Parameter;
    use crate::step_item::CartesianPoint;
    use crate::step_item_map::StepItems;

    #[test]
    fn axis2_placement_3d_from_simple() {
        let se = SimpleEntity {
            keyword: "AXIS2_PLACEMENT_3D".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Reference(1),
                Parameter::Reference(2),
                Parameter::Reference(3),
            ],
        };
        let ap = Axis2Placement3D::from_simple(se).unwrap();
        assert_eq!(ap.location, 1);
        assert_eq!(ap.axis, Some(2));
        assert_eq!(ap.ref_direction, Some(3));
    }

    #[test]
    fn axis2_placement_3d_from_simple_default_axis_ref_direction() {
        let se = SimpleEntity {
            keyword: "AXIS2_PLACEMENT_3D".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Reference(1),
                Parameter::Null,
                Parameter::Null,
            ],
        };
        let ap = Axis2Placement3D::from_simple(se).unwrap();
        assert_eq!(ap.location, 1);
        assert!(ap.axis.is_none());
        assert!(ap.ref_direction.is_none());
    }

    #[test]
    fn axis2_placement_3d_from_simple_not_reference() {
        let se = SimpleEntity {
            keyword: "AXIS2_PLACEMENT_3D".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Reference(1),
                Parameter::String("invalid".into()), // Invalid reference
                Parameter::Null,
            ],
        };
        let err = Axis2Placement3D::from_simple(se).unwrap_err();
        assert!(matches!(
            err,
            ConversionStepItemError::NotReferenceOrNull { .. }
        ));
    }

    #[test]
    fn axis2_placement_3d_from_simple_wrong_keyword() {
        let se = SimpleEntity {
            keyword: "WRONG_KEYWORD".into(),
            attrs: vec![
                Parameter::String("''".into()),
                Parameter::Reference(1),
                Parameter::Reference(2),
                Parameter::Reference(3),
            ],
        };
        let err = Axis2Placement3D::from_simple(se);
        assert!(matches!(
            err,
            Err(ConversionStepItemError::Unsupported { .. })
        ));
    }

    #[test]
    fn axis2_placement_3d_from_simple_attr_count() {
        let se = SimpleEntity {
            keyword: "AXIS2_PLACEMENT_3D".into(),
            attrs: vec![Parameter::String("''".into()), Parameter::Reference(1)],
        };
        let err = Axis2Placement3D::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::AttrCount { expected, found, .. } if expected == 4 && found == 2)
        );
    }

    #[test]
    fn axis2_placement_3d_validate_refs() {
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(
                CartesianPoint {
                    coords: Vector3::new(0.0, 0.0, 0.0),
                }
                .into(),
            ),
        );
        arena.insert(
            2,
            StepItems::new_with_one_item(
                Direction {
                    vec: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            ),
        );
        arena.insert(
            3,
            StepItems::new_with_one_item(
                Direction {
                    vec: Vector3::new(4.0, 5.0, 6.0),
                }
                .into(),
            ),
        );

        let ap = Axis2Placement3D {
            location: 1,
            axis: Some(2),
            ref_direction: Some(3),
        };

        assert!(ap.validate_refs(&arena).is_ok());
    }

    #[test]
    fn axis2_placement_3d_validate_refs_default_axis_ref_direction() {
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(
                CartesianPoint {
                    coords: Vector3::new(0.0, 0.0, 0.0),
                }
                .into(),
            ),
        );

        let ap = Axis2Placement3D {
            location: 1,
            axis: None,          // Default axis
            ref_direction: None, // Default ref_direction
        };

        assert!(ap.validate_refs(&arena).is_ok());
    }

    #[test]
    fn axis2_placement_3d_validate_refs_parallel_axis_ref_direction() {
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(
                CartesianPoint {
                    coords: Vector3::new(0.0, 0.0, 0.0),
                }
                .into(),
            ),
        );
        arena.insert(
            2,
            StepItems::new_with_one_item(
                Direction {
                    vec: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            ),
        );
        arena.insert(
            3,
            StepItems::new_with_one_item(
                Direction {
                    vec: Vector3::new(2.0, 4.0, 6.0), // Parallel to axis
                }
                .into(),
            ),
        );

        let ap = Axis2Placement3D {
            location: 1,
            axis: Some(2),
            ref_direction: Some(3),
        };

        let err = ap.validate_refs(&arena);
        assert!(matches!(
            err,
            Err(ConversionStepItemError::AxisRefDirectionNotOrthogonal)
        ));
    }

    #[test]
    fn axis2_placement_3d_validate_refs_missing_location() {
        let mut arena = StepItemMap::new();
        arena.insert(
            2,
            StepItems::new_with_one_item(
                Direction {
                    vec: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(),
            ),
        );
        arena.insert(
            3,
            StepItems::new_with_one_item(
                Direction {
                    vec: Vector3::new(4.0, 5.0, 6.0),
                }
                .into(),
            ),
        );

        let ap = Axis2Placement3D {
            location: 999, // Unresolved
            axis: Some(2),
            ref_direction: Some(3),
        };

        let err = ap.validate_refs(&arena).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::UnresolvedRef { id } if id == 999));
    }

    #[test]
    fn axis2_placement_3d_wrong_type() {
        let mut arena = StepItemMap::new();
        arena.insert(
            1,
            StepItems::new_with_one_item(
                CartesianPoint {
                    coords: Vector3::new(0.0, 0.0, 0.0),
                }
                .into(),
            ),
        );
        arena.insert(
            2,
            StepItems::new_with_one_item(
                CartesianPoint {
                    coords: Vector3::new(1.0, 2.0, 3.0),
                }
                .into(), // Wrong type
            ),
        );

        let ap = Axis2Placement3D {
            location: 1,
            axis: Some(2), // Should be Direction
            ref_direction: None,
        };

        let err = ap.validate_refs(&arena).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::TypeMismatch { expected, found, id } if expected == "DIRECTION" && found == "CARTESIAN_POINT" && id == 2)
        );
    }
}
