use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntity {
    pub id: usize,
    pub keyword: String,
    pub params: String,
}

static SIMPLE_RE: OnceLock<Regex> = OnceLock::new();
static COMPLEX_RE: OnceLock<Regex> = OnceLock::new();

fn simple_re() -> &'static Regex {
    SIMPLE_RE.get_or_init(|| Regex::new(r"^#(\d+)\s*=\s*([A-Z0-9_]+)\((.*)\);$").unwrap())
}
fn complex_re() -> &'static Regex {
    COMPLEX_RE.get_or_init(|| Regex::new(r"^#(\d+)\s*=\s*\((.*)\);$").unwrap())
}

/// “#…;” で終わる 1 エンティティをパース
pub fn parse_entity(buf: &str) -> Option<RawEntity> {
    if let Some(caps) = simple_re().captures(buf) {
        Some(RawEntity {
            id: caps[1].parse().ok()?,
            keyword: caps[2].to_string(),
            params: caps[3].to_string(),
        })
    } else if let Some(caps) = complex_re().captures(buf) {
        let body = caps[2].trim();
        let kw_end = body.find('(').unwrap_or(body.len());
        Some(RawEntity {
            id: caps[1].parse().ok()?,
            keyword: body[..kw_end].trim().to_string(),
            params: body.to_string(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_entity_simple() {
        let raw = "#1 = AXIS2_PLACEMENT_3D('', (#2,#3,#4));";
        let entity = parse_entity(raw).unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.keyword, "AXIS2_PLACEMENT_3D");
        assert_eq!(entity.params, "'', (#2,#3,#4)");
    }

    #[test]
    fn parse_entity_complex() {
        let raw = "#2 = DUMMY('', (#2,#3,#4,(#2,3.,4.111,.F.,.T.,*,$,'a'),1.1));";
        let entity = parse_entity(raw).unwrap();
        assert_eq!(entity.id, 2);
        assert_eq!(entity.keyword, "DUMMY");
        assert_eq!(entity.params, "'', (#2,#3,#4,(#2,3.,4.111,.F.,.T.,*,$,'a'),1.1)");
    }
}
