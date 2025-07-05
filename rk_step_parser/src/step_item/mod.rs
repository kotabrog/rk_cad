mod common;
mod geometry;
mod topology;

pub use common::{ConversionStepItemError, FromSimple, ValidateRefs};
pub use geometry::{Axis2Placement3D, CartesianPoint, Direction, Line, Plane, Vector};
pub use topology::{EdgeCurve, OrientedEdge, VertexPoint};

use super::step_entity::SimpleEntity;
use super::step_item_map::StepItemMap;

#[derive(Debug)]
pub enum StepItem {
    Direction(Box<Direction>),
    CartesianPoint(Box<CartesianPoint>),
    Vector(Box<Vector>),
    Axis2Placement3D(Box<Axis2Placement3D>),
    VertexPoint(Box<VertexPoint>),
    Line(Box<Line>),
    Plane(Box<Plane>),
    EdgeCurve(Box<EdgeCurve>),
    OrientedEdge(Box<OrientedEdge>),
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
            "VERTEX_POINT" => Ok(StepItem::VertexPoint(Box::new(VertexPoint::from_simple(
                se,
            )?))),
            "LINE" => Ok(StepItem::Line(Box::new(Line::from_simple(se)?))),
            "PLANE" => Ok(StepItem::Plane(Box::new(Plane::from_simple(se)?))),
            "EDGE_CURVE" => Ok(StepItem::EdgeCurve(Box::new(EdgeCurve::from_simple(se)?))),
            "ORIENTED_EDGE" => Ok(StepItem::OrientedEdge(Box::new(OrientedEdge::from_simple(
                se,
            )?))),
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
            StepItem::VertexPoint(_) => "VERTEX_POINT",
            StepItem::Line(_) => "LINE",
            StepItem::Plane(_) => "PLANE",
            StepItem::EdgeCurve(_) => "EDGE_CURVE",
            StepItem::OrientedEdge(_) => "ORIENTED_EDGE",
        }
    }

    pub fn validate_refs(&self, arena: &StepItemMap) -> Result<(), ConversionStepItemError> {
        match self {
            StepItem::Direction(_) => Ok(()),
            StepItem::CartesianPoint(_) => Ok(()),
            StepItem::Vector(vec) => vec.validate_refs(arena),
            StepItem::Axis2Placement3D(ap3d) => ap3d.validate_refs(arena),
            StepItem::VertexPoint(vp) => vp.validate_refs(arena),
            StepItem::Line(line) => line.validate_refs(arena),
            StepItem::Plane(plane) => plane.validate_refs(arena),
            StepItem::EdgeCurve(edge_curve) => edge_curve.validate_refs(arena),
            StepItem::OrientedEdge(oriented_edge) => oriented_edge.validate_refs(arena),
        }
    }
}
