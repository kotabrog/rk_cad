use super::super::{
    as_bool, as_id, as_id_opt, expect_keyword, expect_token_count, fmt_step_bool, fmt_step_opt_id,
    params_list, StepEntity, StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct OrientedEdge {
    pub edge_start: Option<usize>, // -> VERTEX_POINT
    pub edge_end: Option<usize>,   // 同上
    pub edge_curve: usize,         // -> EDGE_CURVE
    pub orientation: bool,
}

impl StepEntity for OrientedEdge {
    const KEYWORD: &'static str = "ORIENTED_EDGE";
}

impl StepParse for OrientedEdge {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , $start , $end , #curve , .T.
        let p = params_list(e);
        expect_token_count(&p, 4, &e.params)?;
        Ok(Self {
            edge_start: as_id_opt(p[0])?,
            edge_end: as_id_opt(p[1])?,
            edge_curve: as_id(p[2])?,
            orientation: as_bool(p[3])?,
        })
    }
}

impl StepWrite for OrientedEdge {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!(
                "'', {}, {}, #{}, {}",
                fmt_step_opt_id(self.edge_start),
                fmt_step_opt_id(self.edge_end),
                self.edge_curve,
                fmt_step_bool(self.orientation)
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oriented_edge_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "ORIENTED_EDGE".into(),
            params: "'', *, *, #99, .T.".into(),
        };
        let edge = OrientedEdge::parse(&raw).unwrap();
        assert_eq!(edge.edge_start, None);
        assert_eq!(edge.edge_end, None);
        assert_eq!(edge.edge_curve, 99);
        assert!(edge.orientation);
    }

    #[test]
    fn oriented_edge_roundtrip() {
        let e1 = OrientedEdge {
            edge_start: None,
            edge_end: None,
            edge_curve: 99,
            orientation: true,
        };
        let raw = OrientedEdge::to_raw(&e1, 42).unwrap();
        let e2 = OrientedEdge::parse(&raw).unwrap();
        assert_eq!(e1, e2);
    }
}
