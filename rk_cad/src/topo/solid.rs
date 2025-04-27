use super::{Face, Shell, TopologyError};

#[derive(Debug, Clone)]
pub struct Solid {
    /// Solid の識別子
    pub id: usize,
    /// 最外殻（必ず 1 つ）
    outer: Shell,
    /// 空洞を表す内殻（0 個以上）
    inners: Vec<Shell>,
}

impl Solid {
    /// チェックなしビルダー
    pub fn new_unchecked(id: usize, outer: Shell, inners: Vec<Shell>) -> Self {
        Solid { id, outer, inners }
    }

    /// 検証付きビルダー
    ///
    /// - 外殻・内殻ともに `Shell::new` でマニホールド検証済みとする  
    /// - （簡易実装）外殻と内殻が「同じ Shell ID」でないかだけを確認  
    ///   ※ 本格的な “包含関係” 判定は今後の拡張ポイント
    pub fn new(id: usize, outer: Shell, inners: Vec<Shell>) -> Result<Self, TopologyError> {
        for sh in &inners {
            if sh.id() == outer.id() {
                return Err(TopologyError::InnerShellSameAsOuter(sh.id()));
            }
        }
        Ok(Solid { id, outer, inners })
    }

    /// 外殻を借用
    pub fn outer(&self) -> &Shell {
        &self.outer
    }

    /// 内殻一覧を借用
    pub fn inners(&self) -> &[Shell] {
        &self.inners
    }

    /// Solid 内部のすべての Face を列挙
    pub fn faces(&self) -> impl Iterator<Item = &Face> {
        self.outer
            .faces()
            .iter()
            .chain(self.inners.iter().flat_map(|sh| sh.faces()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        geo::{AnySurface, PlaneSurface},
        topo::{Edge, OrientedEdge, Vertex, Wire},
    };
    use rk_calc::Vector3;

    /// 1×1×1 立方体を 1 つの Shell → Solid にまとめるテスト
    #[test]
    fn cube_solid() {
        // ────────────── 頂点 ──────────────
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(0.0, 0.0, 1.0));
        let v3 = Vertex::new(3, Vector3::new(0.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Vector3::new(0.0, 1.0, 1.0));
        let v5 = Vertex::new(5, Vector3::new(1.0, 0.0, 0.0));
        let v6 = Vertex::new(6, Vector3::new(1.0, 0.0, 1.0));
        let v7 = Vertex::new(7, Vector3::new(1.0, 1.0, 0.0));
        let v8 = Vertex::new(8, Vector3::new(1.0, 1.0, 1.0));

        // ────────────── 12 エッジ ──────────────
        let e1 = Edge::new_line(1, &v1, &v2).unwrap();
        let e2 = Edge::new_line(2, &v2, &v4).unwrap();
        let e3 = Edge::new_line(3, &v4, &v3).unwrap();
        let e4 = Edge::new_line(4, &v3, &v1).unwrap();
        let e5 = Edge::new_line(5, &v5, &v6).unwrap();
        let e6 = Edge::new_line(6, &v6, &v8).unwrap();
        let e7 = Edge::new_line(7, &v8, &v7).unwrap();
        let e8 = Edge::new_line(8, &v7, &v5).unwrap();
        let e9 = Edge::new_line(9, &v1, &v5).unwrap();
        let e10 = Edge::new_line(10, &v2, &v6).unwrap();
        let e11 = Edge::new_line(11, &v3, &v7).unwrap();
        let e12 = Edge::new_line(12, &v4, &v8).unwrap();

        // ────────────── 6 ループ（外から見て CCW）─────────────
        let left_loop = Wire::new(vec![
            OrientedEdge::new(e1.clone(), true),
            OrientedEdge::new(e2.clone(), true),
            OrientedEdge::new(e3.clone(), true),
            OrientedEdge::new(e4.clone(), true),
        ])
        .unwrap()
        .build_loop(1)
        .unwrap();

        let right_loop = Wire::new(vec![
            OrientedEdge::new(e5.clone(), true),
            OrientedEdge::new(e6.clone(), true),
            OrientedEdge::new(e7.clone(), true),
            OrientedEdge::new(e8.clone(), true),
        ])
        .unwrap()
        .build_loop(2)
        .unwrap();

        let top_loop = Wire::new(vec![
            OrientedEdge::new(e10.clone(), true),
            OrientedEdge::new(e6.clone(), true), // ★ 修正済み
            OrientedEdge::new(e12.clone(), false),
            OrientedEdge::new(e2.clone(), false),
        ])
        .unwrap()
        .build_loop(3)
        .unwrap();

        let bottom_loop = Wire::new(vec![
            OrientedEdge::new(e4.clone(), false),
            OrientedEdge::new(e11.clone(), true),
            OrientedEdge::new(e8.clone(), true),
            OrientedEdge::new(e9.clone(), false),
        ])
        .unwrap()
        .build_loop(4)
        .unwrap();

        let front_loop = Wire::new(vec![
            OrientedEdge::new(e9.clone(), true),
            OrientedEdge::new(e5.clone(), true),
            OrientedEdge::new(e10.clone(), false),
            OrientedEdge::new(e1.clone(), false),
        ])
        .unwrap()
        .build_loop(5)
        .unwrap();

        let back_loop = Wire::new(vec![
            OrientedEdge::new(e3.clone(), false),
            OrientedEdge::new(e12.clone(), true),
            OrientedEdge::new(e7.clone(), true),
            OrientedEdge::new(e11.clone(), false),
        ])
        .unwrap()
        .build_loop(6)
        .unwrap();

        // ────────────── 各面の曲面 ──────────────
        let left_surf: AnySurface = PlaneSurface::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        )
        .unwrap()
        .into();
        let right_surf: AnySurface = PlaneSurface::new(
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        )
        .unwrap()
        .into();
        let top_surf: AnySurface = PlaneSurface::new(
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(0.0, 0.0, 1.0),
            Vector3::new(1.0, 0.0, 0.0),
        )
        .unwrap()
        .into();
        let bottom_surf: AnySurface = PlaneSurface::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(1.0, 0.0, 0.0),
        )
        .unwrap()
        .into();
        let front_surf: AnySurface = PlaneSurface::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        )
        .unwrap()
        .into();
        let back_surf: AnySurface = PlaneSurface::new(
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
        )
        .unwrap()
        .into();

        // ────────────── Face 生成 ──────────────
        let f_left = Face::new(1, left_loop, vec![], left_surf).unwrap();
        let f_right = Face::new(2, right_loop, vec![], right_surf).unwrap();
        let f_top = Face::new(3, top_loop, vec![], top_surf).unwrap();
        let f_bottom = Face::new(4, bottom_loop, vec![], bottom_surf).unwrap();
        let f_front = Face::new(5, front_loop, vec![], front_surf).unwrap();
        let f_back = Face::new(6, back_loop, vec![], back_surf).unwrap();

        // ────────────── Shell → Solid ──────────────
        let outer_shell = Shell::new(1, vec![f_left, f_right, f_top, f_bottom, f_front, f_back])
            .expect("shell should be manifold");

        let solid = Solid::new(1, outer_shell, Vec::new())
            .expect("solid should build with no inner shells");

        assert_eq!(solid.outer().faces().len(), 6);
        assert!(solid.inners().is_empty());
        assert_eq!(solid.faces().count(), 6);
    }
}
