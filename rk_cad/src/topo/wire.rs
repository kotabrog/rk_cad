use super::{OrientedEdge, TopoError};

/// ───────────────────────────────────────────
/// 開いたエッジ列：Wire
/// ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Wire {
    edges: Vec<OrientedEdge>,
}

impl Wire {
    /// 1) 無チェックで生成
    pub fn new_unchecked(edges: Vec<OrientedEdge>) -> Self {
        Wire { edges }
    }

    /// 2) 隣接性チェック付きで生成
    ///    window(2) で連続ペアを走査し、共有頂点があるか確認
    pub fn new(edges: Vec<OrientedEdge>) -> Result<Self, TopoError> {
        for pair in edges.windows(2) {
            if pair[0].end_id() != pair[1].start_id() {
                return Err(TopoError::EdgesNotContiguous);
            }
        }
        Ok(Wire { edges })
    }

    /// 無チェック push
    pub fn unchecked_push(&mut self, oe: OrientedEdge) {
        self.edges.push(oe);
    }

    /// 隣接性チェック付き push
    pub fn checked_push(&mut self, oe: OrientedEdge) -> Result<(), TopoError> {
        if self.edges.is_empty() || self.edges.last().unwrap().end_id() == oe.start_id() {
            self.edges.push(oe);
            Ok(())
        } else {
            Err(TopoError::EdgesNotContiguous)
        }
    }

    /// この Wire が閉じているか（最初の start == 最後の end）
    pub fn is_closed(&self) -> bool {
        if self.edges.is_empty() {
            return false;
        }
        self.edges.first().unwrap().start_id() == self.edges.last().unwrap().end_id()
    }

    /// 閉じていれば Loop を生成、そうでなければ Err
    pub fn build_loop(self, id: usize) -> Result<Loop, TopoError> {
        if self.is_closed() {
            Ok(Loop {
                id,
                edges: self.edges,
            })
        } else {
            Err(TopoError::WireNotClosed)
        }
    }

    /// 読み取り用：全エッジを返す
    pub fn edges(&self) -> &[OrientedEdge] {
        &self.edges
    }
}

/// ───────────────────────────────────────────
/// 閉じたループ：Loop
/// ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Loop {
    pub id: usize,
    edges: Vec<OrientedEdge>,
}

impl Loop {
    /// 一意 ID を取得
    pub fn id(&self) -> usize {
        self.id
    }

    /// ループ上の向き付きエッジ列を読み出し
    pub fn edges(&self) -> &[OrientedEdge] {
        &self.edges
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Edge, Vertex};
    use super::*;
    use rk_calc::Point3;

    #[test]
    fn wire_new() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Point3::new(1.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Point3::new(0.0, 1.0, 0.0));

        let e1 = Edge::new(0, &v1, &v2).unwrap();
        let e2 = Edge::new(1, &v2, &v3).unwrap();
        let e3 = Edge::new(2, &v3, &v4).unwrap();
        let e4 = Edge::new(3, &v4, &v1).unwrap();

        let oe1 = OrientedEdge::new(e1.clone(), true);
        let oe2 = OrientedEdge::new(e2.clone(), true);
        let oe3 = OrientedEdge::new(e3.clone(), true);
        let oe4 = OrientedEdge::new(e4.clone(), true);

        let wire = Wire::new(vec![oe1.clone(), oe2.clone(), oe3.clone(), oe4.clone()]).unwrap();
        assert_eq!(wire.edges().len(), 4);
        let edges = wire.edges();
        assert_eq!(edges[0], oe1);
        assert_eq!(edges[1], oe2);
        assert_eq!(edges[2], oe3);
        assert_eq!(edges[3], oe4);
    }

    #[test]
    #[should_panic(expected = "EdgesNotContiguous")]
    fn wire_new_not_contiguous() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Point3::new(1.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Point3::new(0.0, 1.0, 0.0));

        let e1 = Edge::new(0, &v1, &v2).unwrap();
        let e2 = Edge::new(1, &v2, &v3).unwrap();
        let e3 = Edge::new(2, &v4, &v1).unwrap(); // v4 -> v1 は連続していない

        let oe1 = OrientedEdge::new(e1.clone(), true);
        let oe2 = OrientedEdge::new(e2.clone(), true);
        let oe3 = OrientedEdge::new(e3.clone(), true);

        Wire::new(vec![oe1, oe2, oe3]).unwrap();
    }

    #[test]
    fn checked_push() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Point3::new(1.0, 1.0, 0.0));

        let e1 = Edge::new(0, &v1, &v2).unwrap();
        let e2 = Edge::new(1, &v2, &v3).unwrap();

        let oe1 = OrientedEdge::new(e1.clone(), true);
        let oe2 = OrientedEdge::new(e2.clone(), true);

        let mut wire = Wire::new(vec![oe1]).unwrap();
        assert!(wire.checked_push(oe2).is_ok());
        assert_eq!(wire.edges().len(), 2);
    }

    #[test]
    #[should_panic(expected = "EdgesNotContiguous")]
    fn checked_push_not_contiguous() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Point3::new(1.0, 1.0, 0.0));

        let e1 = Edge::new(0, &v1, &v2).unwrap();
        let e2 = Edge::new(1, &v3, &v2).unwrap(); // v3 -> v2 は連続していない

        let oe1 = OrientedEdge::new(e1.clone(), true);
        let oe2 = OrientedEdge::new(e2.clone(), true);

        let mut wire = Wire::new(vec![oe1]).unwrap();
        wire.checked_push(oe2).unwrap();
    }

    #[test]
    fn is_closed() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Point3::new(1.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Point3::new(0.0, 1.0, 0.0));

        let e1 = Edge::new(0, &v1, &v2).unwrap();
        let e2 = Edge::new(1, &v2, &v3).unwrap();
        let e3 = Edge::new(2, &v3, &v4).unwrap();
        let e4 = Edge::new(3, &v4, &v1).unwrap();

        let oe1 = OrientedEdge::new(e1.clone(), true);
        let oe2 = OrientedEdge::new(e2.clone(), true);
        let oe3 = OrientedEdge::new(e3.clone(), true);
        let oe4 = OrientedEdge::new(e4.clone(), true);

        let wire = Wire::new(vec![oe1, oe2, oe3, oe4]).unwrap();
        assert!(wire.is_closed());
    }

    #[test]
    fn is_closed_not() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Point3::new(1.0, 1.0, 0.0));

        let e1 = Edge::new(0, &v1, &v2).unwrap();
        let e2 = Edge::new(1, &v2, &v3).unwrap();

        let oe1 = OrientedEdge::new(e1.clone(), true);
        let oe2 = OrientedEdge::new(e2.clone(), true);

        let wire = Wire::new(vec![oe1, oe2]).unwrap();
        assert!(!wire.is_closed());
    }

    #[test]
    fn build_loop() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Point3::new(1.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Point3::new(0.0, 1.0, 0.0));

        let e1 = Edge::new(0, &v1, &v2).unwrap();
        let e2 = Edge::new(1, &v2, &v3).unwrap();
        let e3 = Edge::new(2, &v3, &v4).unwrap();
        let e4 = Edge::new(3, &v4, &v1).unwrap();

        let oe1 = OrientedEdge::new(e1.clone(), true);
        let oe2 = OrientedEdge::new(e2.clone(), true);
        let oe3 = OrientedEdge::new(e3.clone(), true);
        let oe4 = OrientedEdge::new(e4.clone(), true);

        let wire = Wire::new(vec![oe1, oe2, oe3, oe4]).unwrap();
        let loop_ = wire.clone().build_loop(42).unwrap();
        assert_eq!(loop_.id(), 42);
        assert_eq!(loop_.edges(), wire.edges());
    }

    #[test]
    #[should_panic(expected = "WireNotClosed")]
    fn build_loop_not_closed() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 0.0, 0.0));
        let v3 = Vertex::new(3, Point3::new(1.0, 1.0, 0.0));

        let e1 = Edge::new(0, &v1, &v2).unwrap();
        let e2 = Edge::new(1, &v2, &v3).unwrap();

        let oe1 = OrientedEdge::new(e1.clone(), true);
        let oe2 = OrientedEdge::new(e2.clone(), true);

        let wire = Wire::new(vec![oe1, oe2]).unwrap();
        wire.build_loop(42).unwrap();
    }
}
