use super::super::geo::{AnySurface, Surface};
use super::{Loop, TopologyError};

/// ───────────────────────────────────────────
/// Face（面）
/// ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Face {
    id: usize,
    /// 外部ループ：必ず閉じていることを前提
    outer: Loop,
    /// 内部ループ（穴）：それぞれ必ず閉じていることを前提
    inners: Vec<Loop>,
    /// この Face が乗っている曲面
    surface: AnySurface,
}

impl Face {
    ///  単一の Loop が与えられた Surface 上にあるか検証
    fn validate_loop_on_surface(
        loop_: &Loop,
        surface: &AnySurface,
        eps: f64,
    ) -> Result<(), TopologyError> {
        for oe in loop_.edges() {
            // OrientedEdge から Edge → Vertex → 座標を取得
            let vtx = oe.edge.v1();
            let p = &vtx.point();
            if !surface.contains_point(p, eps) {
                return Err(TopologyError::VertexNotOnSurface(vtx.id()));
            }
        }
        Ok(())
    }

    /// 新しい Face を生成
    /// Loop 型を受け取るので、各ループが閉じていることは
    /// 既に保証されています。
    ///
    /// # Errors
    /// - `TopologyError::VertexNotOnSurface`: Loop 上の頂点が Surface 上にない
    pub fn new(
        id: usize,
        outer: Loop,
        inners: Vec<Loop>,
        surface: AnySurface,
    ) -> Result<Self, TopologyError> {
        const EPS: f64 = 1e-6;
        Self::validate_loop_on_surface(&outer, &surface, EPS)?;
        for inner in &inners {
            Self::validate_loop_on_surface(inner, &surface, EPS)?;
        }

        Ok(Face {
            id,
            outer,
            inners,
            surface,
        })
    }

    /// Face の一意 ID を取得
    pub fn id(&self) -> usize {
        self.id
    }

    /// 外部ループを借用
    pub fn outer(&self) -> &Loop {
        &self.outer
    }

    /// 内部ループを借用
    pub fn inners(&self) -> &[Loop] {
        &self.inners
    }

    /// 曲面を借用
    pub fn surface(&self) -> &AnySurface {
        &self.surface
    }

    /// 内ループを追加
    pub fn add_inner(&mut self, inner: Loop) -> Result<(), TopologyError> {
        const EPS: f64 = 1e-6;
        Self::validate_loop_on_surface(&inner, &self.surface, EPS)?;
        self.inners.push(inner);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Edge, OrientedEdge, Vertex, Wire};
    use super::*;
    use crate::PlaneSurface;
    use rk_calc::Vector3;

    #[test]
    fn face_new() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Vector3::new(1.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Vector3::new(0.0, 1.0, 0.0));

        let e1 = Edge::new_line(1, &v1, &v2).unwrap();
        let e2 = Edge::new_line(2, &v2, &v3).unwrap();
        let e3 = Edge::new_line(3, &v3, &v4).unwrap();
        let e4 = Edge::new_line(4, &v4, &v1).unwrap();

        let wire = Wire::new_unchecked(vec![
            OrientedEdge::new(e1.clone(), true),
            OrientedEdge::new(e2.clone(), true),
            OrientedEdge::new(e3.clone(), true),
            OrientedEdge::new(e4.clone(), true),
        ]);
        let loop_outer = wire.build_loop(0).unwrap();

        let surface: AnySurface = PlaneSurface::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
        )
        .unwrap()
        .into();

        let face = Face::new(1, loop_outer.clone(), vec![], surface.clone()).unwrap();

        assert_eq!(face.id(), 1);
        assert_eq!(face.outer().id(), loop_outer.id);
        assert!(face.inners().is_empty());
        assert_eq!(face.surface(), &surface);
    }

    #[test]
    #[should_panic(expected = "VertexNotOnSurface")]
    fn face_new_invalid() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Vector3::new(1.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Vector3::new(0.0, 1.0, 0.0));

        let e1 = Edge::new_line(1, &v1, &v2).unwrap();
        let e2 = Edge::new_line(2, &v2, &v3).unwrap();
        let e3 = Edge::new_line(3, &v3, &v4).unwrap();
        let e4 = Edge::new_line(4, &v4, &v1).unwrap();

        let wire_outer = Wire::new_unchecked(vec![
            OrientedEdge::new(e1.clone(), true),
            OrientedEdge::new(e2.clone(), true),
            OrientedEdge::new(e3.clone(), true),
            OrientedEdge::new(e4.clone(), true),
        ]);
        let loop_outer = wire_outer.build_loop(0).unwrap();

        let surface: AnySurface = PlaneSurface::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
        )
        .unwrap()
        .into();

        // Loop 上の頂点が Surface 上にない
        let v5 = Vertex::new(5, Vector3::new(10.0, 10.0, 10.0));
        let v6 = Vertex::new(6, Vector3::new(11.0, 10.0, 10.0));
        let v7 = Vertex::new(7, Vector3::new(11.0, 11.0, 10.0));

        let e5 = Edge::new_line(5, &v5, &v6).unwrap();
        let e6 = Edge::new_line(6, &v6, &v7).unwrap();
        let e7 = Edge::new_line(7, &v7, &v5).unwrap();

        let wire_inner = Wire::new_unchecked(vec![
            OrientedEdge::new(e5.clone(), true),
            OrientedEdge::new(e6.clone(), true),
            OrientedEdge::new(e7.clone(), true),
        ]);
        let loop_inner = wire_inner.build_loop(1).unwrap();

        Face::new(1, loop_outer.clone(), vec![loop_inner], surface).unwrap();
    }

    #[test]
    fn face_add_inner() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Vector3::new(1.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Vector3::new(0.0, 1.0, 0.0));

        let e1 = Edge::new_line(1, &v1, &v2).unwrap();
        let e2 = Edge::new_line(2, &v2, &v3).unwrap();
        let e3 = Edge::new_line(3, &v3, &v4).unwrap();
        let e4 = Edge::new_line(4, &v4, &v1).unwrap();

        let wire_outer = Wire::new_unchecked(vec![
            OrientedEdge::new(e1.clone(), true),
            OrientedEdge::new(e2.clone(), true),
            OrientedEdge::new(e3.clone(), true),
            OrientedEdge::new(e4.clone(), true),
        ]);
        let loop_outer = wire_outer.build_loop(0).unwrap();

        let surface: AnySurface = PlaneSurface::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
        )
        .unwrap()
        .into();

        let mut face = Face::new(1, loop_outer.clone(), vec![], surface).unwrap();

        let v5 = Vertex::new(5, Vector3::new(0.5, 0.5, 0.0));
        let v6 = Vertex::new(6, Vector3::new(0.75, 0.5, 0.0));
        let v7 = Vertex::new(7, Vector3::new(0.75, 0.75, 0.0));
        let v8 = Vertex::new(8, Vector3::new(0.5, 0.75, 0.0));
        let e5 = Edge::new_line(5, &v5, &v6).unwrap();
        let e6 = Edge::new_line(6, &v6, &v7).unwrap();
        let e7 = Edge::new_line(7, &v7, &v8).unwrap();
        let e8 = Edge::new_line(8, &v8, &v5).unwrap();
        let wire_inner = Wire::new_unchecked(vec![
            OrientedEdge::new(e5.clone(), true),
            OrientedEdge::new(e6.clone(), true),
            OrientedEdge::new(e7.clone(), true),
            OrientedEdge::new(e8.clone(), true),
        ]);
        let loop_inner = wire_inner.build_loop(1).unwrap();
        face.add_inner(loop_inner.clone()).unwrap();
        assert_eq!(face.inners().len(), 1);
        assert_eq!(face.inners()[0].id, loop_inner.id);
        assert_eq!(face.inners()[0].edges().len(), 4);
    }

    #[test]
    #[should_panic(expected = "VertexNotOnSurface")]
    fn face_add_inner_invalid() {
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Vector3::new(1.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Vector3::new(0.0, 1.0, 0.0));

        let e1 = Edge::new_line(1, &v1, &v2).unwrap();
        let e2 = Edge::new_line(2, &v2, &v3).unwrap();
        let e3 = Edge::new_line(3, &v3, &v4).unwrap();
        let e4 = Edge::new_line(4, &v4, &v1).unwrap();

        let wire_outer = Wire::new_unchecked(vec![
            OrientedEdge::new(e1.clone(), true),
            OrientedEdge::new(e2.clone(), true),
            OrientedEdge::new(e3.clone(), true),
            OrientedEdge::new(e4.clone(), true),
        ]);
        let loop_outer = wire_outer.build_loop(0).unwrap();

        let surface: AnySurface = PlaneSurface::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
        )
        .unwrap()
        .into();

        let mut face = Face::new(1, loop_outer.clone(), vec![], surface).unwrap();

        // Loop 上の頂点が Surface 上にない
        let v5 = Vertex::new(5, Vector3::new(10.0, 10.0, 10.0));
        let v6 = Vertex::new(6, Vector3::new(11.0, 10.0, 10.0));
        let v7 = Vertex::new(7, Vector3::new(11.0, 11.0, 10.0));
        let v8 = Vertex::new(8, Vector3::new(10.0, 11.0, 10.0));
        let e5 = Edge::new_line(5, &v5, &v6).unwrap();
        let e6 = Edge::new_line(6, &v6, &v7).unwrap();
        let e7 = Edge::new_line(7, &v7, &v8).unwrap();
        let e8 = Edge::new_line(8, &v8, &v5).unwrap();
        let wire_inner = Wire::new_unchecked(vec![
            OrientedEdge::new(e5.clone(), true),
            OrientedEdge::new(e6.clone(), true),
            OrientedEdge::new(e7.clone(), true),
            OrientedEdge::new(e8.clone(), true),
        ]);
        let loop_inner = wire_inner.build_loop(1).unwrap();
        face.add_inner(loop_inner.clone()).unwrap();
    }
}
