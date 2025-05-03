use super::super::{
    as_id, expect_keyword, expect_token_count, params_list, StepEntity, StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct VertexPoint {
    pub point_id: usize,
}

impl StepEntity for VertexPoint {
    const KEYWORD: &'static str = "VERTEX_POINT";
}

impl StepParse for VertexPoint {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , #123
        let tok = params_list(e);
        expect_token_count(&tok, 1, &e.params)?;
        let point_id = as_id(tok[0])?;
        Ok(VertexPoint { point_id })
    }
}

impl StepWrite for VertexPoint {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!("'', #{}", self.point_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_point_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "VERTEX_POINT".into(),
            params: "'', #99".into(),
        };
        let vertex = VertexPoint::parse(&raw).unwrap();
        assert_eq!(vertex.point_id, 99);
    }

    #[test]
    fn vertex_point_roundtrip() {
        let v1 = VertexPoint { point_id: 99 };
        let raw = VertexPoint::to_raw(&v1, 42).unwrap();
        let v2 = VertexPoint::parse(&raw).unwrap();
        assert_eq!(v1, v2);
    }
}
