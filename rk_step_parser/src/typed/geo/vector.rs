use super::super::{
    as_f64, as_id, expect_keyword, expect_token_count, fmt_step_real, params_list, StepEntity,
    StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    pub dir_id: usize,
    pub magnitude: f64,
}

impl StepEntity for Vector {
    const KEYWORD: &'static str = "VECTOR";
}

impl StepParse for Vector {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , #d , m
        let tok = params_list(e);
        expect_token_count(&tok, 2, &e.params)?;
        Ok(Vector {
            dir_id: as_id(tok[0])?,
            magnitude: as_f64(tok[1])?,
        })
    }
}

impl StepWrite for Vector {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!("'', #{}, {}", self.dir_id, fmt_step_real(self.magnitude)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "VECTOR".into(),
            params: "'', #1, 2.0".into(),
        };
        let vector = Vector::parse(&raw).unwrap();
        assert_eq!(vector.dir_id, 1);
        assert_eq!(vector.magnitude, 2.0);
    }

    #[test]
    fn vector_roundtrip() {
        let v1 = Vector {
            dir_id: 1,
            magnitude: 2.0,
        };
        let raw = Vector::to_raw(&v1, 42).unwrap();
        let v2 = Vector::parse(&raw).unwrap();
        assert_eq!(v1, v2);
    }
}
