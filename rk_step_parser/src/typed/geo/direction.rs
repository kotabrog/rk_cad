use super::super::{
    as_f64, expect_keyword, expect_token_count, fmt_step_real, params_list, StepEntity, StepParse,
    StepWrite,
};
use crate::{ParseError, RawEntity};
use rk_calc::Vector3;

#[derive(Debug, Clone, PartialEq)]
pub struct Direction(pub Vector3);

impl StepEntity for Direction {
    const KEYWORD: &'static str = "DIRECTION";
}

impl StepParse for Direction {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , (x, y, z)
        let tok = params_list(e);
        expect_token_count(&tok, 3, &e.params)?;

        let coords = Vector3::new(as_f64(tok[0])?, as_f64(tok[1])?, as_f64(tok[2])?);
        Ok(Self(coords))
    }
}

impl StepWrite for Direction {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        let v = self.0;
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!(
                "'', ({}, {}, {})",
                fmt_step_real(v.x)?,
                fmt_step_real(v.y)?,
                fmt_step_real(v.z)?
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direction_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "DIRECTION".into(),
            params: "'', (1.0, 2.0, 3.0)".into(),
        };
        let dir = Direction::parse(&raw).unwrap();
        assert_eq!(dir.0, Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn direction_roundtrip() {
        let d1 = Direction(Vector3::new(1.0, 2.0, 3.0));
        let raw = Direction::to_raw(&d1, 42).unwrap();
        let d2 = Direction::parse(&raw).unwrap();
        assert_eq!(d1, d2);
    }
}
