use super::super::{
    as_id, expect_keyword, fmt_step_id_list, tokenized, StepEntity, StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeLoop {
    pub edges: Vec<usize>, // -> ORIENTED_EDGE
}

impl StepEntity for EdgeLoop {
    const KEYWORD: &'static str = "EDGE_LOOP";
}

impl StepParse for EdgeLoop {
    /// 現在の実装では、()を無視しているため
    /// '' , (#edge1, #edge2, ...) の形でない場合に正常に動作しない可能性がある
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , (#edge1, #edge2, ...)
        Ok(Self {
            edges: tokenized(&e.params)
                .map(as_id)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl StepWrite for EdgeLoop {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!("'', {}", fmt_step_id_list(&self.edges)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_loop_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "EDGE_LOOP".into(),
            params: "'', (#1, #2, #3)".into(),
        };
        let edge_loop = EdgeLoop::parse(&raw).unwrap();
        assert_eq!(edge_loop.edges, vec![1, 2, 3]);
    }

    #[test]
    fn edge_loop_roundtrip() {
        let el1 = EdgeLoop {
            edges: vec![1, 2, 3],
        };
        let raw = EdgeLoop::to_raw(&el1, 42).unwrap();
        let el2 = EdgeLoop::parse(&raw).unwrap();
        assert_eq!(el1, el2);
    }
}
