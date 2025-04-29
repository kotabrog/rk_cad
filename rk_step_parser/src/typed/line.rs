use super::{
    as_id, expect_keyword, expect_token_count, fmt_step_real, tokenized, StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};
use rk_calc::Vector3;

/* ---------- DIRECTION & VECTOR -------------------------------- */

#[derive(Debug, Clone, PartialEq)]
pub struct Direction(pub Vector3);

impl StepWrite for Direction {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        let v = self.0;
        Ok(RawEntity {
            id,
            keyword: "DIRECTION".into(),
            params: format!(
                "'', ({}, {}, {})",
                fmt_step_real(v.x)?,
                fmt_step_real(v.y)?,
                fmt_step_real(v.z)?
            ),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vector {
    pub dir_id: usize,
    pub magnitude: f64,
}

impl StepWrite for Vector {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: "VECTOR".into(),
            params: format!("'', #{}, {}", self.dir_id, fmt_step_real(self.magnitude)?),
        })
    }
}

/* ---------- LINE ---------------------------------------------- */

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub point_id: usize,
    pub vector_id: usize,
}

impl StepParse for Line {
    const KEYWORD: &'static str = "LINE";
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
    fn direction_to_row() {
        // direction
        let answer = RawEntity {
            id: 1,
            keyword: "DIRECTION".into(),
            params: "'', (0., 0., 1.)".into(),
        };
        let dir = Direction(Vector3::new(0.0, 0.0, 1.0));
        let dir_raw = dir.to_raw(1).unwrap();
        assert_eq!(answer, dir_raw);
    }

    #[test]
    fn vector_to_row() {
        // vector
        let raw1 = RawEntity {
            id: 2,
            keyword: "VECTOR".into(),
            params: "'', #1, 1.".into(),
        };
        let vec = Vector {
            dir_id: 1,
            magnitude: 1.0,
        };
        let raw2 = vec.to_raw(2).unwrap();
        assert_eq!(raw1, raw2);
    }

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
