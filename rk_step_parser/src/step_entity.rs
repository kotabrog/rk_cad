//! 1 行の STEP レコード  →  StepEntity へ変換する簡易パーサ
//! ・ISO 10303-21 Edition 3 のデータ部 (§12) に対応
//! ・複合エンティティは `( KEYWORD(..) KEYWORD(..) … )` を Vec<SimpleEntity> へ展開

use std::str::Chars;
use thiserror::Error;

pub type EntityId = usize;

#[derive(Debug)]
pub struct StepEntity {
    pub id: EntityId,
    pub parts: Vec<SimpleEntity>, // ← simple / complex どちらも対応
}

#[derive(Debug, Clone)]
pub struct SimpleEntity {
    pub keyword: String,       // EDGE_LOOP, VERTEX_POINT …
    pub attrs: Vec<Parameter>, // ← エンティティの PARAMETER_LIST
}

/// PARAMETER ─ simple / aggregate / typed / null
#[derive(Debug, Clone)]
pub enum Parameter {
    // simple types (ISO 10303-21 §6.4)
    Integer(i64),
    Real(f64),
    String(String),
    Enumeration(String),   // .MILLI., .TRUE. など
    Logical(Option<bool>), // .T. / .F. / .U.
    Reference(EntityId),   // #123
    Binary(String),        // "ABCD4E" etc.

    // structured types (§7.1)
    Aggregate(Vec<Parameter>), // LIST/SET/ARRAY/BAG 値
    Typed(Box<TypedParameter>),

    // null tokens (§12.2.2)
    Null,    // $  (値なし)
    Omitted, // *  (OPTIONAL 未指定)
}

#[derive(Debug, Clone)]
pub struct TypedParameter {
    pub type_name: String,
    pub inner: Parameter,
}

/* ─────────────── error type ─────────────── */

#[derive(Debug, Error)]
pub enum StepEntityParseError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("unexpected character: {0}")]
    UnexpectedChar(char),
    #[error("invalid number literal: {0}")]
    InvalidNumber(String),
    #[error("invalid entity reference: {0}")]
    InvalidReference(String),
    #[error("{0}")]
    Other(String),
}

/* ─────────────── public API ─────────────── */

/// Parse one STEP data-section record (simple or complex).
pub fn parse_step_entity(line: &str) -> Result<StepEntity, StepEntityParseError> {
    let mut chars = Cursor::new(line);

    chars.skip_ws();
    chars.expect('#')?;
    let id = chars.parse_usize()?;
    chars.skip_ws();
    chars.expect('=')?;
    chars.skip_ws();

    // decide external "( … )" or internal "KEYWORD("…
    let parts = match chars.peek() {
        Some('(') => parse_complex_external(&mut chars)?,
        Some(c) if c.is_ascii_alphabetic() => parse_complex_internal(&mut chars)?,
        Some(c) => return Err(StepEntityParseError::UnexpectedChar(*c)),
        None => return Err(StepEntityParseError::UnexpectedEof),
    };

    chars.skip_ws();
    chars.expect(';')?;
    chars.skip_ws();
    if chars.peek().is_some() {
        return Err(StepEntityParseError::Other("trailing characters".into()));
    }

    Ok(StepEntity { id, parts })
}

/* ─────────────── complex entity helpers ─────────────── */

/// external mapping:  (#id = ( A() B() ) ; )
fn parse_complex_external(chars: &mut Cursor) -> Result<Vec<SimpleEntity>, StepEntityParseError> {
    chars.expect('(')?;
    let mut parts = Vec::new();
    loop {
        chars.skip_ws();
        parts.push(parse_simple_entity(chars)?);
        chars.skip_ws();
        match chars.peek() {
            Some(')') => {
                chars.next();
                break;
            }
            Some(_) => {} // space → next simple entity
            None => return Err(StepEntityParseError::UnexpectedEof),
        }
    }
    Ok(parts)
}

/// internal mapping:  #id = A() B() C();
fn parse_complex_internal(chars: &mut Cursor) -> Result<Vec<SimpleEntity>, StepEntityParseError> {
    let mut parts = Vec::new();
    loop {
        chars.skip_ws();
        parts.push(parse_simple_entity(chars)?);
        chars.skip_ws();
        match chars.peek() {
            Some(';') => break,
            Some(c) if c.is_ascii_alphabetic() => continue, // next keyword
            Some(c) => return Err(StepEntityParseError::UnexpectedChar(*c)),
            None => return Err(StepEntityParseError::UnexpectedEof),
        }
    }
    Ok(parts)
}

/* ─────────────── simple entity & parameters ─────────────── */

fn parse_simple_entity(chars: &mut Cursor) -> Result<SimpleEntity, StepEntityParseError> {
    let keyword = chars.parse_ident()?;
    chars.skip_ws();
    chars.expect('(')?;

    let mut attrs = Vec::new();
    if chars.peek() != Some(&')') {
        loop {
            attrs.push(parse_parameter(chars)?);
            chars.skip_ws();
            match chars.peek() {
                Some(',') => {
                    chars.next();
                    chars.skip_ws();
                }
                Some(')') => break,
                Some(c) => return Err(StepEntityParseError::UnexpectedChar(*c)),
                None => return Err(StepEntityParseError::UnexpectedEof),
            }
        }
    }
    chars.expect(')')?;
    Ok(SimpleEntity { keyword, attrs })
}

fn parse_parameter(chars: &mut Cursor) -> Result<Parameter, StepEntityParseError> {
    chars.skip_ws();
    match chars.peek() {
        Some('\'') => parse_quoted_string(chars),
        Some('#') => {
            chars.next();
            let id = chars.parse_usize()?;
            Ok(Parameter::Reference(id))
        }
        Some('.') => parse_dot_literal(chars),
        Some('(') => parse_aggregate(chars),
        Some('*') => {
            chars.next();
            Ok(Parameter::Omitted)
        }
        Some('$') => {
            chars.next();
            Ok(Parameter::Null)
        }
        Some('"') => parse_binary(chars),
        Some(c) if c.is_ascii_digit() || *c == '-' || *c == '+' => parse_number(chars),
        Some(c) if c.is_ascii_alphabetic() => parse_typed_parameter(chars),
        Some(c) => Err(StepEntityParseError::UnexpectedChar(*c)),
        None => Err(StepEntityParseError::UnexpectedEof),
    }
}

/* ─────────────── leaf literal parsers ─────────────── */

fn parse_aggregate(chars: &mut Cursor) -> Result<Parameter, StepEntityParseError> {
    chars.expect('(')?;
    let mut vals = Vec::new();
    if chars.peek() != Some(&')') {
        loop {
            vals.push(parse_parameter(chars)?);
            chars.skip_ws();
            match chars.peek() {
                Some(',') => {
                    chars.next();
                    chars.skip_ws();
                }
                Some(')') => break,
                Some(c) => return Err(StepEntityParseError::UnexpectedChar(*c)),
                None => return Err(StepEntityParseError::UnexpectedEof),
            }
        }
    }
    chars.expect(')')?;
    Ok(Parameter::Aggregate(vals))
}

fn parse_typed_parameter(chars: &mut Cursor) -> Result<Parameter, StepEntityParseError> {
    let type_name = chars.parse_ident()?;
    chars.skip_ws();
    chars.expect('(')?;
    let inner = parse_parameter(chars)?;
    chars.expect(')')?;
    Ok(Parameter::Typed(Box::new(TypedParameter {
        type_name,
        inner,
    })))
}

fn parse_quoted_string(chars: &mut Cursor) -> Result<Parameter, StepEntityParseError> {
    chars.expect('\'')?;
    let mut s = String::new();
    while let Some(c) = chars.next() {
        match c {
            '\'' => break,
            _ => s.push(c),
        }
    }
    Ok(Parameter::String(s))
}

fn parse_binary(chars: &mut Cursor) -> Result<Parameter, StepEntityParseError> {
    chars.expect('"')?;
    let mut s = String::new();
    while let Some(c) = chars.next() {
        match c {
            '"' => break,
            _ => s.push(c),
        }
    }
    Ok(Parameter::Binary(s))
}

fn parse_dot_literal(chars: &mut Cursor) -> Result<Parameter, StepEntityParseError> {
    chars.expect('.')?;
    let lit = chars.parse_ident()?.to_ascii_uppercase();
    chars.expect('.')?;

    match lit.as_str() {
        "T" | "TRUE" => Ok(Parameter::Logical(Some(true))),
        "F" | "FALSE" => Ok(Parameter::Logical(Some(false))),
        "U" | "UNKNOWN" | "UNDEFINED" => Ok(Parameter::Logical(None)),
        _ => Ok(Parameter::Enumeration(lit)),
    }
}

fn parse_number(chars: &mut Cursor) -> Result<Parameter, StepEntityParseError> {
    let mut buf = String::new();
    while let Some(c) = chars.peek() {
        if c.is_ascii_alphanumeric()
            || *c == '.'
            || *c == '-'
            || *c == '+'
            || *c == 'E'
            || *c == 'e'
        {
            buf.push(*c);
            chars.next();
        } else {
            break;
        }
    }
    if buf.contains('.') || buf.contains('E') || buf.contains('e') {
        buf.parse::<f64>()
            .map(Parameter::Real)
            .map_err(|_| StepEntityParseError::InvalidNumber(buf))
    } else {
        buf.parse::<i64>()
            .map(Parameter::Integer)
            .map_err(|_| StepEntityParseError::InvalidNumber(buf))
    }
}

/* ─────────────── Cursor helper ─────────────── */

struct Cursor<'a> {
    iter: Chars<'a>,
    peeked: Option<Option<char>>,
}

impl<'a> Cursor<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            iter: s.chars(),
            peeked: None,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        if self.peeked.is_none() {
            self.peeked = Some(self.iter.next());
        }
        self.peeked.as_ref().unwrap().as_ref()
    }

    fn next(&mut self) -> Option<char> {
        if let Some(c_opt) = self.peeked.take() {
            c_opt
        } else {
            self.iter.next()
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(c) if c.is_whitespace()) {
            self.next();
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), StepEntityParseError> {
        match self.next() {
            Some(c) if c == ch => Ok(()),
            Some(c) => Err(StepEntityParseError::UnexpectedChar(c)),
            None => Err(StepEntityParseError::UnexpectedEof),
        }
    }

    fn parse_ident(&mut self) -> Result<String, StepEntityParseError> {
        let mut s = String::new();
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() => {}
            Some(c) => return Err(StepEntityParseError::UnexpectedChar(*c)),
            None => return Err(StepEntityParseError::UnexpectedEof),
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || *c == '_' || *c == '-' {
                s.push(*c);
                self.next();
            } else {
                break;
            }
        }
        Ok(s)
    }

    fn parse_usize(&mut self) -> Result<usize, StepEntityParseError> {
        let mut num = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num.push(*c);
                self.next();
            } else {
                break;
            }
        }
        num.parse::<usize>()
            .map_err(|_| StepEntityParseError::InvalidReference(num))
    }
}

/* ─────────────── tests ─────────────── */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_step_entity_simple() {
        let src = "#12 = CARTESIAN_POINT('', (0.0, 0.0, 0.0));";
        let ent = parse_step_entity(src).unwrap();
        assert_eq!(ent.id, 12);
        assert_eq!(ent.parts.len(), 1);
        assert_eq!(ent.parts[0].keyword, "CARTESIAN_POINT");
    }

    #[test]
    fn parse_step_entity_complex_external() {
        let src = "#166 = ( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) );";
        let ent = parse_step_entity(src).unwrap();
        assert_eq!(ent.parts.len(), 3);
        assert_eq!(ent.parts[2].keyword, "SI_UNIT");
    }

    #[test]
    fn parse_step_entity_complex_internal() {
        let src = "#166 = LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.);";
        let ent = parse_step_entity(src).unwrap();
        assert_eq!(ent.parts.len(), 3);
        assert_eq!(ent.parts[0].keyword, "LENGTH_UNIT");
    }

    #[test]
    fn parse_step_entity_typed_param() {
        let src = "#169 = UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-07),#166,'d','c');";
        let ent = parse_step_entity(src).unwrap();
        let outer = &ent.parts[0];
        assert_eq!(outer.keyword, "UNCERTAINTY_MEASURE_WITH_UNIT");
        if let Parameter::Typed(tp) = &outer.attrs[0] {
            assert_eq!(tp.type_name, "LENGTH_MEASURE");
        } else {
            panic!("first attr not typed");
        }
    }

    /* ────────────────────── 各 Parameter パターン ────────────────────── */

    #[test]
    fn parse_step_entity_parameter_integer() {
        let ent = parse_step_entity("#1 = INT_TEST(123);").unwrap();
        matches!(ent.parts[0].attrs[0], Parameter::Integer(123));
    }

    #[test]
    fn parse_step_entity_parameter_real() {
        let ent = parse_step_entity("#2 = REAL_TEST(1.23E+4);").unwrap();
        assert!(matches!(ent.parts[0].attrs[0], Parameter::Real(r) if (r - 12300.0).abs() < 1e-6));
    }

    #[test]
    fn parse_step_entity_parameter_string() {
        let ent = parse_step_entity("#3 = STR_TEST('hello world');").unwrap();
        assert!(matches!(ent.parts[0].attrs[0], Parameter::String(ref s) if s == "hello world"));
    }

    #[test]
    fn parse_step_entity_parameter_enumeration() {
        let ent = parse_step_entity("#4 = ENUM_TEST(.milli.);").unwrap();
        assert!(matches!(ent.parts[0].attrs[0], Parameter::Enumeration(ref e) if e == "MILLI"));
    }

    #[test]
    fn parse_step_entity_parameter_logical() {
        let ent_true = parse_step_entity("#5 = LOG_TEST(.T.);").unwrap();
        let ent_false = parse_step_entity("#6 = LOG_TEST(.FALSE.);").unwrap();
        let ent_unknown = parse_step_entity("#7 = LOG_TEST(.U.);").unwrap();
        assert!(matches!(
            ent_true.parts[0].attrs[0],
            Parameter::Logical(Some(true))
        ));
        assert!(matches!(
            ent_false.parts[0].attrs[0],
            Parameter::Logical(Some(false))
        ));
        assert!(matches!(
            ent_unknown.parts[0].attrs[0],
            Parameter::Logical(None)
        ));
    }

    #[test]
    fn parse_step_entity_parameter_reference() {
        let ent = parse_step_entity("#8 = REF_TEST(#5);").unwrap();
        assert!(matches!(ent.parts[0].attrs[0], Parameter::Reference(5)));
    }

    #[test]
    fn parse_step_entity_parameter_binary() {
        let ent = parse_step_entity("#9 = BIN_TEST(\"ABCD\");").unwrap();
        assert!(matches!(ent.parts[0].attrs[0], Parameter::Binary(ref b) if b == "ABCD"));
    }

    #[test]
    fn parse_step_entity_parameter_aggregate() {
        let ent = parse_step_entity("#10 = LIST_TEST((1,2,3));").unwrap();
        if let Parameter::Aggregate(ref vec) = ent.parts[0].attrs[0] {
            assert_eq!(vec.len(), 3);
            assert!(matches!(vec[0], Parameter::Integer(1)));
        } else {
            panic!("aggregate not parsed");
        }
    }

    #[test]
    fn parse_step_entity_parameter_typed() {
        let ent = parse_step_entity("#11 = TP_TEST(LENGTH_MEASURE(2.0));").unwrap();
        assert!(
            matches!(ent.parts[0].attrs[0], Parameter::Typed(ref tp) if tp.type_name == "LENGTH_MEASURE")
        );
    }

    #[test]
    fn parse_step_entity_parameter_null_and_omitted() {
        let ent = parse_step_entity("#12 = NULL_TEST($,*);").unwrap();
        assert!(matches!(ent.parts[0].attrs[0], Parameter::Null));
        assert!(matches!(ent.parts[0].attrs[1], Parameter::Omitted));
    }

    /* ──────────────────────── エラー発生パターン ─────────────────────── */

    #[test]
    fn parse_step_entity_error_unexpected_eof() {
        let err = parse_step_entity("#20 = CARTESIAN_POINT(").unwrap_err();
        assert!(matches!(err, StepEntityParseError::UnexpectedEof));
    }

    #[test]
    fn parse_step_entity_error_unexpected_char() {
        let err = parse_step_entity("#21 = CPC(@);").unwrap_err();
        assert!(matches!(err, StepEntityParseError::UnexpectedChar('@')));
    }

    #[test]
    fn parse_step_entity_error_invalid_number() {
        let err = parse_step_entity("#22 = NUM_ERR(12A);").unwrap_err();
        assert!(matches!(err, StepEntityParseError::InvalidNumber(ref s) if s == "12A"));
    }

    #[test]
    fn parse_step_entity_error_invalid_reference() {
        let err = parse_step_entity("#23 = REF_ERR(#AB);").unwrap_err();
        assert!(matches!(err, StepEntityParseError::InvalidReference(ref s) if s.is_empty()));
    }

    #[test]
    fn parse_step_entity_error_trailing_characters() {
        let err = parse_step_entity("#25 = CPC('', (0.,0.,0.)); extra").unwrap_err();
        assert!(matches!(err, StepEntityParseError::Other(ref msg) if msg.contains("trailing")));
    }

    /* ───────────────────────── 空白の多いケース ─────────────────────── */

    #[test]
    fn parse_step_entity_parse_with_extra_whitespace() {
        let src = "#30   =   CARTESIAN_POINT   (  ''  ,  (  0.0 , 0.0  , 0.0  )   ) ;  ";
        let ent = parse_step_entity(src).unwrap();
        assert_eq!(ent.parts[0].keyword, "CARTESIAN_POINT");
        assert!(matches!(ent.parts[0].attrs[1], Parameter::Aggregate(_)));
    }
}
