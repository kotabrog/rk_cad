mod common;
mod geometry;

pub use common::{ConversionStepItemError, FromSimple, ValidateRefs};
pub use geometry::{Axis2Placement3D, CartesianPoint, Direction, Vector};

use super::step_entity::SimpleEntity;
use super::step_item_map::StepItemMap;

#[derive(Debug)]
pub enum StepItem {
    Direction(Box<Direction>),
    CartesianPoint(Box<CartesianPoint>),
    Vector(Box<Vector>),
    Axis2Placement3D(Box<Axis2Placement3D>),
}

impl TryFrom<SimpleEntity> for StepItem {
    type Error = ConversionStepItemError;
    fn try_from(se: SimpleEntity) -> Result<Self, Self::Error> {
        match se.keyword.as_str() {
            "DIRECTION" => Ok(StepItem::Direction(Box::new(Direction::from_simple(se)?))),
            "CARTESIAN_POINT" => Ok(StepItem::CartesianPoint(Box::new(
                CartesianPoint::from_simple(se)?,
            ))),
            "VECTOR" => Ok(StepItem::Vector(Box::new(Vector::from_simple(se)?))),
            "AXIS2_PLACEMENT_3D" => Ok(StepItem::Axis2Placement3D(Box::new(
                Axis2Placement3D::from_simple(se)?,
            ))),
            other => Err(ConversionStepItemError::Unsupported(other.into())),
        }
    }
}

impl StepItem {
    pub fn keyword(&self) -> &'static str {
        match self {
            StepItem::Direction(_) => "DIRECTION",
            StepItem::CartesianPoint(_) => "CARTESIAN_POINT",
            StepItem::Vector(_) => "VECTOR",
            StepItem::Axis2Placement3D(_) => "AXIS2_PLACEMENT_3D",
        }
    }

    pub fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        match self {
            StepItem::Direction(_) => Ok(()),
            StepItem::CartesianPoint(_) => Ok(()),
            StepItem::Vector(vec) => vec.validate_refs(arena),
            StepItem::Axis2Placement3D(ap3d) => ap3d.validate_refs(arena),
        }
    }
}
