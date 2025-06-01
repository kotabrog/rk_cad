mod common;
mod geometry;

pub use common::{ConversionStepItemError, FromSimple};
pub use geometry::Direction;

use super::step_entity::SimpleEntity;

#[derive(Debug)]
pub enum StepItem {
    Direction(Box<Direction>),
}

impl TryFrom<SimpleEntity> for StepItem {
    type Error = ConversionStepItemError;
    fn try_from(se: SimpleEntity) -> Result<Self, Self::Error> {
        match se.keyword.as_str() {
            "DIRECTION" => Ok(StepItem::Direction(Box::new(Direction::from_simple(se)?))),
            other => Err(ConversionStepItemError::Unsupported(other.into())),
        }
    }
}
