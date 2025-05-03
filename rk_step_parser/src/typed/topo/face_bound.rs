use super::super::{
    as_bool, as_id, expect_keyword, expect_token_count, fmt_step_bool, params_list, StepEntity,
    StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct FaceBound {
    pub loop_id: usize, // -> EDGE_LOOP
    pub orientation: bool,
}

impl StepEntity for FaceBound {
    const KEYWORD: &'static str = "FACE_BOUND";
}

impl StepParse for FaceBound {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , #loop_id , .T.
        let p = params_list(e);
        expect_token_count(&p, 2, &e.params)?;
        Ok(Self {
            loop_id: as_id(p[0])?,
            orientation: as_bool(p[1])?,
        })
    }
}

impl StepWrite for FaceBound {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!("'', #{}, {}", self.loop_id, fmt_step_bool(self.orientation)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn face_bound_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "FACE_BOUND".into(),
            params: "'', #1, .T.".into(),
        };
        let face_bound = FaceBound::parse(&raw).unwrap();
        assert_eq!(face_bound.loop_id, 1);
        assert!(face_bound.orientation);
    }

    #[test]
    fn face_bound_roundtrip() {
        let fb1 = FaceBound {
            loop_id: 1,
            orientation: true,
        };
        let raw = FaceBound::to_raw(&fb1, 42).unwrap();
        let fb2 = FaceBound::parse(&raw).unwrap();
        assert_eq!(fb1, fb2);
    }
}
