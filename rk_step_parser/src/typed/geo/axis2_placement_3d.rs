use super::super::{
    as_id, as_id_opt, expect_keyword, expect_token_count, fmt_step_opt_id, params_list, StepEntity,
    StepParse, StepWrite,
};
use crate::{ParseError, RawEntity};

#[derive(Debug, Clone, PartialEq)]
pub struct Axis2Placement3D {
    pub location: usize,        // 必須 CARTESIAN_POINT
    pub axis: Option<usize>,    // 省略は $
    pub ref_dir: Option<usize>, // 省略は $
}

impl StepEntity for Axis2Placement3D {
    const KEYWORD: &'static str = "AXIS2_PLACEMENT_3D";
}

impl StepParse for Axis2Placement3D {
    fn parse(e: &RawEntity) -> Result<Self, ParseError> {
        expect_keyword(e, Self::KEYWORD)?;
        // '' , #location, $axis, $ref_dir
        let tok = params_list(e);
        expect_token_count(&tok, 3, &e.params)?;

        let location = as_id(tok[0])?;
        let axis = as_id_opt(tok[1])?;
        let ref_dir = as_id_opt(tok[2])?;
        Ok(Self {
            location,
            axis,
            ref_dir,
        })
    }
}

impl StepWrite for Axis2Placement3D {
    fn to_raw(&self, id: usize) -> Result<RawEntity, ParseError> {
        Ok(RawEntity {
            id,
            keyword: Self::KEYWORD.into(),
            params: format!(
                "'', #{}, {}, {}",
                self.location,
                fmt_step_opt_id(self.axis),
                fmt_step_opt_id(self.ref_dir),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis2_placement_3d_parse_none() {
        let raw = RawEntity {
            id: 42,
            keyword: "AXIS2_PLACEMENT_3D".into(),
            params: "'', #1, $, $".into(),
        };
        let axis = Axis2Placement3D::parse(&raw).unwrap();
        assert_eq!(axis.location, 1);
        assert_eq!(axis.axis, None);
        assert_eq!(axis.ref_dir, None);
    }

    #[test]
    fn axis2_placement_3d_roundtrip_none() {
        let a1 = Axis2Placement3D {
            location: 1,
            axis: None,
            ref_dir: None,
        };
        let raw = Axis2Placement3D::to_raw(&a1, 42).unwrap();
        let a2 = Axis2Placement3D::parse(&raw).unwrap();
        assert_eq!(a1, a2);
    }

    #[test]
    fn axis2_placement_3d_parse_some() {
        let raw = RawEntity {
            id: 42,
            keyword: "AXIS2_PLACEMENT_3D".into(),
            params: "'', #1, #2, #3".into(),
        };
        let axis = Axis2Placement3D::parse(&raw).unwrap();
        assert_eq!(axis.location, 1);
        assert_eq!(axis.axis, Some(2));
        assert_eq!(axis.ref_dir, Some(3));
    }

    #[test]
    fn axis2_placement_3d_roundtrip_some() {
        let a1 = Axis2Placement3D {
            location: 1,
            axis: Some(2),
            ref_dir: Some(3),
        };
        let raw = Axis2Placement3D::to_raw(&a1, 42).unwrap();
        let a2 = Axis2Placement3D::parse(&raw).unwrap();
        assert_eq!(a1, a2);
    }
}
