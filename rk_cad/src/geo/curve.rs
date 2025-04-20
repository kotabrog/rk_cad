use rk_calc::Vector3;

/// Curve の共通インターフェース
pub trait Curve: std::fmt::Debug + Clone + PartialEq {
    /// パラメータ t (通常 0.0..1.0) における位置を返す
    fn position(&self, t: f64) -> Vector3;
    /// パラメータ t における接線ベクトルを返す
    fn tangent(&self, t: f64) -> Vector3;
    /// t = 0.0 の位置を返す
    fn start(&self) -> Vector3 {
        self.position(0.0)
    }
    /// t = 1.0 の位置を返す
    fn end(&self) -> Vector3 {
        self.position(1.0)
    }
}

/// 線分を表す最も基本的な Curve
#[derive(Debug, Clone, PartialEq)]
pub struct LineCurve {
    pub start: Vector3,
    pub end: Vector3,
}

impl LineCurve {
    /// 新しい線分を生成
    pub fn new(start: Vector3, end: Vector3) -> Self {
        LineCurve { start, end }
    }
}

impl Curve for LineCurve {
    fn position(&self, t: f64) -> Vector3 {
        // (1 - t)*start + t*end
        self.start + (self.end - self.start) * t
    }

    fn tangent(&self, _t: f64) -> Vector3 {
        // 線分方向を常に同じ接線とみなす
        (self.end - self.start).normalize()
    }
}

/// 将来の円弧やスプラインなどを追加するための enum
#[derive(Debug, Clone, PartialEq)]
pub enum AnyCurve {
    Line(LineCurve),
    // Circle(CircleCurve),
    // Nurbs(NurbsCurve),
}

impl Curve for AnyCurve {
    fn position(&self, t: f64) -> Vector3 {
        match self {
            AnyCurve::Line(l) => l.position(t),
            // AnyCurve::Circle(c) => c.position(t),
            // …
        }
    }
    fn tangent(&self, t: f64) -> Vector3 {
        match self {
            AnyCurve::Line(l) => l.tangent(t),
            // …
        }
    }
}

impl From<LineCurve> for AnyCurve {
    fn from(line: LineCurve) -> Self {
        AnyCurve::Line(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rk_calc::Vector3;

    #[test]
    fn line_curve() {
        let start = Vector3::new(0.0, 0.0, 0.0);
        let end = Vector3::new(1.0, 1.0, 1.0);
        let line = LineCurve::new(start, end);

        assert_eq!(line.position(0.0), start);
        assert_eq!(line.position(1.0), end);
        assert_eq!(line.position(0.5), Vector3::new(0.5, 0.5, 0.5));
        assert_eq!(line.tangent(0.5), Vector3::new(1.0, 1.0, 1.0).normalize());
    }

    #[test]
    fn any_curve() {
        let start = Vector3::new(0.0, 0.0, 0.0);
        let end = Vector3::new(1.0, 1.0, 1.0);
        let line = AnyCurve::Line(LineCurve::new(start, end));

        assert_eq!(line.position(0.0), start);
        assert_eq!(line.position(1.0), end);
        assert_eq!(line.position(0.5), Vector3::new(0.5, 0.5, 0.5));
        assert_eq!(line.tangent(0.5), Vector3::new(1.0, 1.0, 1.0).normalize());
    }
}
