//! ISO 10303-42 ― ENTITY `CARTESIAN_POINT` （2021 版）
//!
//! ```exp
//! ENTITY cartesian_point
//!   --  Subtype chain: representation_item -> geometric_representation_item
//!   --                     -> point -> cartesian_point
//!   SUPERTYPE OF (ONEOF(cylindrical_point, polar_point, spherical_point));
//!   SUBTYPE OF (point);
//!     coordinates : LIST [1:3] OF length_measure; -- 明示属性
//! END_ENTITY;
//!
//! 交換ファイル例:
//!   #42 = CARTESIAN_POINT('', (0., 0., 0.));
//!
//! * `coordinates` リスト長 = 座標次元 (1,2,3)。AP203/214/242 では 3 要素が標準。
//! * **INTEGER リテラルも受理し `f64` へ昇格する**（例: `(1, 2, 3)` → `(1.0, 2.0, 3.0)`).  
//! -----------------------------------------------------------------------------
//! Implementation notes:
//! - 3 要素 → `Vector3` として保持
//! - 2 要素 → `ConversionStepItemError::TwoDimUnsupported`
//! - 1 要素 → `ConversionStepItemError::OneDimUnsupported`
//! - 4 要素以上 → `ConversionStepItemError::ItemCountExceeded`

use super::super::common::{
    aggregate_to_f64, check_keyword, expect_attr_len, ConversionStepItemError, FromSimple,
};
use super::super::StepItem;
use crate::step_entity::SimpleEntity;
use rk_calc::Vector3;

/// # CARTESIAN_POINT
/// 直交座標点（x, y, z）を表す。  
/// - 2 D / 1 D は未対応（仕様上は有効）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartesianPoint {
    pub coords: Vector3,
}

impl FromSimple for CartesianPoint {
    const KEYWORD: &'static str = "CARTESIAN_POINT";

    fn from_simple(se: SimpleEntity) -> Result<Self, ConversionStepItemError> {
        check_keyword(&se, Self::KEYWORD)?;

        // name, coordinates の 2 属性を期待
        expect_attr_len(&se, 2, Self::KEYWORD)?;

        // 2 番目の属性が座標リスト
        let vals = aggregate_to_f64(&se.attrs[1], Self::KEYWORD)?;

        match vals.len() {
            3 => Ok(Self {
                coords: Vector3::new(vals[0], vals[1], vals[2]),
            }),
            2 => Err(ConversionStepItemError::TwoDimUnsupported {
                keyword: Self::KEYWORD,
            }),
            1 => Err(ConversionStepItemError::OneDimUnsupported {
                keyword: Self::KEYWORD,
            }),
            n => Err(ConversionStepItemError::ItemCount {
                keyword: Self::KEYWORD,
                expected_min: 3,
                expected_max: 3,
                found: n,
            }),
        }
    }
}

impl From<CartesianPoint> for StepItem {
    fn from(cp: CartesianPoint) -> Self {
        StepItem::CartesianPoint(cp.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step_entity::Parameter;

    #[test]
    fn cartesian_point_from_simple() {
        let se = SimpleEntity {
            keyword: "CARTESIAN_POINT".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Aggregate(vec![
                    Parameter::Real(0.0),
                    Parameter::Real(1.0),
                    Parameter::Real(2.0),
                ]),
            ],
        };
        let cp = CartesianPoint::from_simple(se).unwrap();
        assert_eq!(cp.coords, Vector3::new(0.0, 1.0, 2.0));
    }

    #[test]
    fn cartesian_point_from_simple_2d() {
        let se = SimpleEntity {
            keyword: "CARTESIAN_POINT".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Aggregate(vec![Parameter::Real(0.0), Parameter::Real(1.0)]),
            ],
        };
        let err = CartesianPoint::from_simple(se).unwrap_err();
        assert!(matches!(
            err,
            ConversionStepItemError::TwoDimUnsupported { .. }
        ));
    }

    #[test]
    fn cartesian_point_from_simple_1d() {
        let se = SimpleEntity {
            keyword: "CARTESIAN_POINT".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Aggregate(vec![Parameter::Real(0.0)]),
            ],
        };
        let err = CartesianPoint::from_simple(se).unwrap_err();
        assert!(matches!(
            err,
            ConversionStepItemError::OneDimUnsupported { .. }
        ));
    }

    #[test]
    fn cartesian_point_from_simple_too_many() {
        let se = SimpleEntity {
            keyword: "CARTESIAN_POINT".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Aggregate(vec![
                    Parameter::Real(0.0),
                    Parameter::Real(1.0),
                    Parameter::Real(2.0),
                    Parameter::Real(3.0),
                ]),
            ],
        };
        let err = CartesianPoint::from_simple(se).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::ItemCount { .. }));
    }

    #[test]
    fn cartesian_point_from_simple_empty() {
        let se = SimpleEntity {
            keyword: "CARTESIAN_POINT".into(),
            attrs: vec![Parameter::String("".into())],
        };
        let err = CartesianPoint::from_simple(se).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::AttrCount { .. }));
    }

    #[test]
    fn cartesian_point_from_simple_non_numeric() {
        let se = SimpleEntity {
            keyword: "CARTESIAN_POINT".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Aggregate(vec![Parameter::String("invalid".into())]),
            ],
        };
        let err = CartesianPoint::from_simple(se).unwrap_err();
        assert!(
            matches!(err, ConversionStepItemError::NonNumeric { keyword } if keyword == "CARTESIAN_POINT")
        );
    }

    #[test]
    fn cartesian_point_from_simple_invalid_keyword() {
        let se = SimpleEntity {
            keyword: "INVALID_POINT".into(),
            attrs: vec![
                Parameter::String("".into()),
                Parameter::Aggregate(vec![Parameter::Real(0.0), Parameter::Real(1.0)]),
            ],
        };
        let err = CartesianPoint::from_simple(se).unwrap_err();
        assert!(matches!(err, ConversionStepItemError::Unsupported(_)));
    }
}
