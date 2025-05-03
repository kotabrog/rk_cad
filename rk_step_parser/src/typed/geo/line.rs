use super::super::{
    as_id, expect_keyword, expect_token_count, tokenized, StepEntity, StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub point_id: usize,
    pub vector_id: usize,
}

impl StepEntity for Line {
    const KEYWORD: &'static str = "LINE";
}

impl StepParse for Line {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , #p , #v
        let tok = tokenized(&e.params).collect::<Vec<_>>();
        expect_token_count(&tok, 2, &e.params)?;
        let point_id = as_id(tok[0])?;
        let vector_id = as_id(tok[1])?;
        Ok(Line {
            point_id,
            vector_id,
        })
    }
}

impl StepWrite for Line {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!("'', #{}, #{}", self.point_id, self.vector_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_parse() {
        let raw = RawEntity {
            id: 3,
            keyword: "LINE".into(),
            params: "'', #1, #2".into(),
        };
        let line = Line::parse(&raw).unwrap();
        assert_eq!(line.point_id, 1);
        assert_eq!(line.vector_id, 2);
    }

    #[test]
    fn line_roundtrip() {
        let l1 = Line {
            point_id: 1,
            vector_id: 2,
        };
        let raw = l1.to_raw(3).unwrap();
        let l2 = Line::parse(&raw).unwrap();
        assert_eq!(l1, l2);
    }
}
