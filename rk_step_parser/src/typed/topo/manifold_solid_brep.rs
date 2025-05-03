use super::super::{
    as_id, expect_keyword, expect_token_count, params_list, StepEntity, StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct ManifoldSolidBrep {
    pub shell: usize,
} // -> CLOSED_SHELL

impl StepEntity for ManifoldSolidBrep {
    const KEYWORD: &'static str = "MANIFOLD_SOLID_BREP";
}

impl StepParse for ManifoldSolidBrep {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , #shell
        let tok = params_list(e);
        expect_token_count(&tok, 1, &e.params)?;
        Ok(Self {
            shell: as_id(tok[0])?,
        })
    }
}

impl StepWrite for ManifoldSolidBrep {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!("'', #{}", self.shell),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifold_solid_brep_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "MANIFOLD_SOLID_BREP".into(),
            params: "'', #99".into(),
        };
        let manifold = ManifoldSolidBrep::parse(&raw).unwrap();
        assert_eq!(manifold.shell, 99);
    }

    #[test]
    fn manifold_solid_brep_roundtrip() {
        let m1 = ManifoldSolidBrep { shell: 99 };
        let raw = ManifoldSolidBrep::to_raw(&m1, 42).unwrap();
        let m2 = ManifoldSolidBrep::parse(&raw).unwrap();
        assert_eq!(m1, m2);
    }
}
