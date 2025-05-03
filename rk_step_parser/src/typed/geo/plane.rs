use super::super::{
    as_id, expect_keyword, expect_token_count, tokenized, StepEntity, StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    pub axis2_id: usize,
}

impl StepEntity for Plane {
    const KEYWORD: &'static str = "PLANE";
}

impl StepParse for Plane {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , #axis
        let tok = tokenized(&e.params).collect::<Vec<_>>();
        expect_token_count(&tok, 1, &e.params)?;
        let axis2_id = as_id(tok[0])?;
        Ok(Plane { axis2_id })
    }
}

impl StepWrite for Plane {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!("'', #{}", self.axis2_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "PLANE".into(),
            params: "'', #99".into(),
        };
        let plane = Plane::parse(&raw).unwrap();
        assert_eq!(plane.axis2_id, 99);
    }

    #[test]
    fn plane_roundtrip() {
        let p1 = Plane { axis2_id: 99 };
        let raw = Plane::to_raw(&p1, 42).unwrap();
        let p2 = Plane::parse(&raw).unwrap();
        assert_eq!(p1, p2);
    }
}
