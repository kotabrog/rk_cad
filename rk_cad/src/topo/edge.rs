use super::{TopologyError, Vertex};
use crate::geo::{AnyCurve, LineCurve};
use rk_calc::Vector3;
use std::{
    cell::{Ref, RefCell, RefMut},
    fmt,
    ops::Deref,
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeData {
    id: usize,
    v1: Vertex,
    v2: Vertex,
    curve: AnyCurve,
}

impl EdgeData {
    /// 新しい EdgeData を生成。
    /// v1.id == v2.id の場合は Err を返す。
    fn new(id: usize, v1: &Vertex, v2: &Vertex, curve: AnyCurve) -> Result<Self, TopologyError> {
        if v1.id() == v2.id() {
            return Err(TopologyError::EdgeEndpointsEqual);
        }
        Ok(EdgeData {
            id,
            v1: v1.clone(),
            v2: v2.clone(),
            curve,
        })
    }
}

/// Rc<RefCell<EdgeData>> をラップした型
#[derive(Clone, Debug, PartialEq)]
pub struct Edge(Rc<RefCell<EdgeData>>);

impl Edge {
    /// 新しい Edge を生成。
    /// 同一頂点を両端に指定した場合は Err(TopologyError::EdgeEndpointsEqual)。
    pub fn new<C>(id: usize, v1: &Vertex, v2: &Vertex, curve: C) -> Result<Self, TopologyError>
    where
        C: Into<AnyCurve>,
    {
        let data = EdgeData::new(id, v1, v2, curve.into())?;
        Ok(Edge(Rc::new(RefCell::new(data))))
    }

    /// 線分 Edge の簡易ビルダー
    pub fn new_line(id: usize, v1: &Vertex, v2: &Vertex) -> Result<Self, TopologyError> {
        let curve = AnyCurve::Line(LineCurve::new(v1.point(), v2.point()));
        Self::new(id, v1, v2, curve)
    }

    /// ID を取得
    pub fn id(&self) -> usize {
        self.0.borrow().id
    }

    /// 端点１を取得
    pub fn v1(&self) -> Vertex {
        self.0.borrow().v1.clone()
    }

    /// 端点２を取得
    pub fn v2(&self) -> Vertex {
        self.0.borrow().v2.clone()
    }

    /// 曲線を取得
    pub fn curve(&self) -> AnyCurve {
        self.0.borrow().curve.clone()
    }

    /// 内部データへの不変借用
    pub fn borrow(&self) -> Ref<'_, EdgeData> {
        self.0.borrow()
    }

    /// 内部データへの可変借用
    pub fn borrow_mut(&self) -> RefMut<'_, EdgeData> {
        self.0.borrow_mut()
    }

    /// Edge 上の曲線を借用
    pub fn borrow_curve(&self) -> Ref<'_, AnyCurve> {
        Ref::map(self.0.borrow(), |d| &d.curve)
    }

    /// 向き付きエッジを生成
    pub fn to_oriented_edge(&self, forward: bool) -> OrientedEdge {
        OrientedEdge::new(self.clone(), forward)
    }

    /// Edge の長さ
    pub fn length(&self) -> f64 {
        let d = self.0.borrow();
        d.v1.distance(&d.v2)
    }

    /// 両端頂点を平行移動
    pub fn translate_endpoints(&self, delta: Vector3) {
        let d = self.0.borrow_mut();
        d.v1.set_point(d.v1.point() + delta);
        d.v2.set_point(d.v2.point() + delta);
    }
}

/// 向き付きエッジを表す補助型
#[derive(Clone, PartialEq)]
pub struct OrientedEdge {
    /// 実際の Edge
    pub edge: Edge,
    /// true: v1→v2、false: v2→v1
    pub forward: bool,
}

impl OrientedEdge {
    /// 新しい OrientedEdge を生成
    pub fn new(edge: Edge, forward: bool) -> Self {
        OrientedEdge { edge, forward }
    }

    /// この向き付きエッジの始点 Vertex ID
    pub fn start_id(&self) -> usize {
        if self.forward {
            self.edge.v1().id()
        } else {
            self.edge.v2().id()
        }
    }

    /// この向き付きエッジの終点 Vertex ID
    pub fn end_id(&self) -> usize {
        if self.forward {
            self.edge.v2().id()
        } else {
            self.edge.v1().id()
        }
    }
}

impl fmt::Debug for OrientedEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (s, e) = (self.start_id(), self.end_id());
        f.debug_struct("OrientedEdge")
            .field("edge_id", &self.edge.id())
            .field("forward", &self.forward)
            .field("start_id", &s)
            .field("end_id", &e)
            .finish()
    }
}

/// OrientedEdge を &Edge に deref coercion させる
impl Deref for OrientedEdge {
    type Target = Edge;

    fn deref(&self) -> &Self::Target {
        &self.edge
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::Curve;
    use rk_calc::Vector3;

    #[test]
    fn edge_new() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let curve = LineCurve::new(v1.point(), v2.point());
        let edge = Edge::new(1, &v1, &v2, curve.clone());
        assert!(edge.is_ok());
        let edge = edge.unwrap();
        assert_eq!(edge.id(), 1);
        assert_eq!(edge.v1().id(), 1);
        assert_eq!(edge.v2().id(), 2);
        assert_eq!(edge.curve(), curve.into());
    }

    #[test]
    fn edge_new_line() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        assert_eq!(edge.id(), 1);
        assert_eq!(edge.v1().id(), 1);
        assert_eq!(edge.v2().id(), 2);
        assert_eq!(
            edge.curve(),
            AnyCurve::Line(LineCurve::new(v1.point(), v2.point()))
        );
    }

    #[test]
    fn edge_borrow() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        let borrowed_edge = edge.borrow();
        assert_eq!(borrowed_edge.id, 1);
        assert_eq!(borrowed_edge.v1.id(), 1);
        assert_eq!(borrowed_edge.v2.id(), 2);
    }

    #[test]
    fn edge_borrow_mut() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        let mut borrowed_edge = edge.borrow_mut();
        borrowed_edge.id = 2;
        assert_eq!(borrowed_edge.id, 2);
    }

    #[test]
    fn edge_borrow_curve() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        let borrowed_curve = edge.borrow_curve();
        assert_eq!(borrowed_curve.start(), v1.point());
        assert_eq!(borrowed_curve.end(), v2.point());
    }

    #[test]
    fn edge_length() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(3.0, 4.0, 0.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        assert_eq!(edge.length(), 5.0);
    }

    #[test]
    fn edge_translate_endpoints() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        edge.translate_endpoints(Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(edge.v1().point().x, 1.0);
        assert_eq!(edge.v1().point().y, 2.0);
        assert_eq!(edge.v1().point().z, 3.0);
        assert_eq!(edge.v2().point().x, 2.0);
        assert_eq!(edge.v2().point().y, 3.0);
        assert_eq!(edge.v2().point().z, 4.0);
    }

    #[test]
    fn oriented_edge_new() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        let oriented_edge = OrientedEdge::new(edge.clone(), true);
        assert_eq!(oriented_edge.start_id(), 1);
        assert_eq!(oriented_edge.end_id(), 2);
    }

    #[test]
    fn to_oriented_edge() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        let oriented_edge = edge.to_oriented_edge(true);
        assert_eq!(oriented_edge.start_id(), 1);
        assert_eq!(oriented_edge.end_id(), 2);
    }

    #[test]
    fn oriented_edge_debug() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        let oriented_edge = OrientedEdge::new(edge.clone(), true);
        assert_eq!(
            format!("{:?}", oriented_edge),
            "OrientedEdge { edge_id: 1, forward: true, start_id: 1, end_id: 2 }"
        );
    }

    #[test]
    fn oriented_edge_deref() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 1.0, 1.0));
        let edge = Edge::new_line(1, &v1, &v2).unwrap();
        let oriented_edge = OrientedEdge::new(edge.clone(), true);
        assert_eq!(oriented_edge.id(), 1);
        assert_eq!(oriented_edge.v1().id(), 1);
        assert_eq!(oriented_edge.v2().id(), 2);
    }
}
