// rk_step_parser/src/typed/cartesian_point.rs
use super::super::{
    as_f64, expect_keyword, expect_token_count, fmt_step_real, params_list, StepEntity, StepParse,
    StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct CartesianPoint {
    pub coords: [f64; 3],
}

impl StepEntity for CartesianPoint {
    const KEYWORD: &'static str = "CARTESIAN_POINT";
}

impl StepParse for CartesianPoint {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , (x, y, z)
        let tok = params_list(e);
        expect_token_count(&tok, 3, &e.params)?;
        Ok(Self {
            coords: [as_f64(tok[0])?, as_f64(tok[1])?, as_f64(tok[2])?],
        })
    }
}

impl StepWrite for CartesianPoint {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!(
                "'', {}, {}, {}",
                fmt_step_real(self.coords[0])?,
                fmt_step_real(self.coords[1])?,
                fmt_step_real(self.coords[2])?
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cartesian_point_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "CARTESIAN_POINT".into(),
            params: "'', 1.0, 2.0, 3.0".into(),
        };
        let point = CartesianPoint::parse(&raw).unwrap();
        assert_eq!(point.coords, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn cartesian_point_roundtrip() {
        let p1 = CartesianPoint {
            coords: [1.0, 2.0, 3.0],
        };
        let raw = CartesianPoint::to_raw(&p1, 42).unwrap();
        let p2 = CartesianPoint::parse(&raw).unwrap();
        assert_eq!(p1, p2);
    }
}
