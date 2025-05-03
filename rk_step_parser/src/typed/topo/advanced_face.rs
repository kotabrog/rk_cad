use super::super::{
    as_bool, as_id, expect_keyword, expect_token_count_min, fmt_step_bool, fmt_step_id_list,
    params_list, StepEntity, StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancedFace {
    pub bounds: Vec<usize>, // -> FACE_BOUND
    pub surface: usize,     // -> PLANE / NURBS など (geom)
    pub orientation: bool,
}

impl StepEntity for AdvancedFace {
    const KEYWORD: &'static str = "ADVANCED_FACE";
}

impl StepParse for AdvancedFace {
    /// 現在の実装では括弧を無視するので
    /// '' , ( #loop_id , #loop_id , ... ) , #surface , .T.の形でない場合に予期せぬ動作をする可能性がある
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , ( #loop_id , #loop_id , ... ) , #surface , .T.
        let p = params_list(e);
        expect_token_count_min(&p, 3, &e.params)?;
        let (bounds, surface, orient) = (&p[..p.len() - 2], p[p.len() - 2], p[p.len() - 1]);
        let bounds = bounds
            .iter()
            .map(|t| as_id(t))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            bounds,
            surface: as_id(surface)?,
            orientation: as_bool(orient)?,
        })
    }
}

impl StepWrite for AdvancedFace {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!(
                "'' , {} , #{} , {}",
                fmt_step_id_list(&self.bounds),
                self.surface,
                fmt_step_bool(self.orientation),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advanced_face_parse() {
        let raw = RawEntity {
            id: 42,
            keyword: "ADVANCED_FACE".into(),
            params: "'', (#1, #2, #3), #4, .T.".into(),
        };
        let advanced_face = AdvancedFace::parse(&raw).unwrap();
        assert_eq!(advanced_face.bounds, vec![1, 2, 3]);
        assert_eq!(advanced_face.surface, 4);
        assert!(advanced_face.orientation);
    }

    #[test]
    fn advanced_face_roundtrip() {
        let af1 = AdvancedFace {
            bounds: vec![1, 2, 3],
            surface: 4,
            orientation: true,
        };
        let raw = AdvancedFace::to_raw(&af1, 42).unwrap();
        let af2 = AdvancedFace::parse(&raw).unwrap();
        assert_eq!(af1, af2);
    }
}
