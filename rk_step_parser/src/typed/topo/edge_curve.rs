use super::super::{
    as_bool, as_id, expect_keyword, expect_token_count, fmt_step_bool, params_list, StepEntity,
    StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeCurve {
    pub v1: usize,
    pub v2: usize,
    pub curve_id: usize,
    pub same_sense: bool,
}

impl StepEntity for EdgeCurve {
    const KEYWORD: &'static str = "EDGE_CURVE";
}

impl StepParse for EdgeCurve {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;

        // '' , #v1 , #v2 , #curve_id , .T.
        let tok = params_list(e);
        expect_token_count(&tok, 4, &e.params)?;
        let v1 = as_id(tok[0])?;
        let v2 = as_id(tok[1])?;
        let curve_id = as_id(tok[2])?;
        let same_sense = as_bool(tok[3])?;

        Ok(EdgeCurve {
            v1,
            v2,
            curve_id,
            same_sense,
        })
    }
}

impl StepWrite for EdgeCurve {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!(
                "'', #{}, #{}, #{}, {}",
                self.v1,
                self.v2,
                self.curve_id,
                fmt_step_bool(self.same_sense),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_curve_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "EDGE_CURVE".into(),
            params: "'', #1, #2, #3, .T.".into(),
        };
        let ec = EdgeCurve::parse(&raw).unwrap();
        assert_eq!(ec.v1, 1);
        assert_eq!(ec.v2, 2);
        assert_eq!(ec.curve_id, 3);
        assert!(ec.same_sense);
    }

    #[test]
    fn edge_curve_roundtrip() {
        let ec1 = EdgeCurve {
            v1: 1,
            v2: 2,
            curve_id: 3,
            same_sense: true,
        };
        let raw = ec1.to_raw(42).unwrap();
        let ec2 = EdgeCurve::parse(&raw).unwrap();
        assert_eq!(ec1, ec2);
    }
}
