use super::super::{
    as_id, expect_keyword, fmt_step_id_list, tokenized, StepEntity, StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct ClosedShell {
    pub faces: Vec<usize>,
}

impl StepEntity for ClosedShell {
    const KEYWORD: &'static str = "CLOSED_SHELL";
}

impl StepParse for ClosedShell {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , (#face1, #face2, ...)
        Ok(Self {
            faces: tokenized(&e.params)
                .map(as_id)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl StepWrite for ClosedShell {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!("'', {}", fmt_step_id_list(&self.faces)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_shell_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "CLOSED_SHELL".into(),
            params: "'', (#1, #2, #3)".into(),
        };
        let closed_shell = ClosedShell::parse(&raw).unwrap();
        assert_eq!(closed_shell.faces, vec![1, 2, 3]);
    }

    #[test]
    fn closed_shell_roundtrip() {
        let cs1 = ClosedShell {
            faces: vec![1, 2, 3],
        };
        let raw = ClosedShell::to_raw(&cs1, 42).unwrap();
        let cs2 = ClosedShell::parse(&raw).unwrap();
        assert_eq!(cs1, cs2);
    }
}
