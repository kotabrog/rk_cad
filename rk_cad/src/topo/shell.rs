use super::{Face, TopologyError};
use std::collections::HashMap;

/// ───────────────────────────────────────────
/// Shell（面の集合体）
/// ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Shell {
    /// シェル ID
    pub id: usize,
    /// このシェルを構成する Face の一覧
    faces: Vec<Face>,
}

impl Shell {
    /// チェックなしで生成（面同士の接合チェックは行わない）
    pub fn new_unchecked(id: usize, faces: Vec<Face>) -> Self {
        Shell { id, faces }
    }

    /// 接合性チェック付きで生成
    ///
    /// 全 Face の外部ループ・内ループ上のすべての Edge が
    /// ちょうど２回ずつ現れる（＝各エッジが２面に共有される）か検証します。
    pub fn new(id: usize, faces: Vec<Face>) -> Result<Self, TopologyError> {
        // Edge ID ごとの出現回数を数える
        let mut count: HashMap<usize, usize> = HashMap::new();

        for face in &faces {
            // 外部ループ
            for oe in face.outer().edges() {
                *count.entry(oe.edge.id()).or_default() += 1;
            }
            // 内部ループ（孔）
            for inner in face.inners() {
                for oe in inner.edges() {
                    *count.entry(oe.edge.id()).or_default() += 1;
                }
            }
        }

        // 出現回数チェック
        for (edge_id, &cnt) in &count {
            if cnt != 2 {
                return Err(TopologyError::ShellNotManifoldEdge(*edge_id, cnt));
            }
        }

        Ok(Shell { id, faces })
    }

    /// シェルを構成する Face を借用
    pub fn faces(&self) -> &[Face] {
        &self.faces
    }

    /// シェルを消費して Face の Vec を取り出し
    pub fn into_faces(self) -> Vec<Face> {
        self.faces
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

    /// 立方体 1×1×1 の 6 面を 1 Shell にまとめるテスト
    #[test]
    fn cube_shell() {
        // ────────────── 頂点 ──────────────
        let v1 = Vertex::new(1, Vector3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Vector3::new(0.0, 0.0, 1.0));
        let v3 = Vertex::new(3, Vector3::new(0.0, 1.0, 0.0));
        let v4 = Vertex::new(4, Vector3::new(0.0, 1.0, 1.0));
        let v5 = Vertex::new(5, Vector3::new(1.0, 0.0, 0.0));
        let v6 = Vertex::new(6, Vector3::new(1.0, 0.0, 1.0));
        let v7 = Vertex::new(7, Vector3::new(1.0, 1.0, 0.0));
        let v8 = Vertex::new(8, Vector3::new(1.0, 1.0, 1.0));

        // ────────────── 12 本のエッジ ──────────────
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

        // ────────────── 6 面のワイヤ（外から見て CCW）──────────────
        // 左面 (x=0)
        let left_loop = Wire::new(vec![
            OrientedEdge::new(e1.clone(), true), // v1→v2
            OrientedEdge::new(e2.clone(), true), // v2→v4
            OrientedEdge::new(e3.clone(), true), // v4→v3
            OrientedEdge::new(e4.clone(), true), // v3→v1
        ])
        .unwrap()
        .build_loop(1)
        .unwrap();

        // 右面 (x=1)
        let right_loop = Wire::new(vec![
            OrientedEdge::new(e5.clone(), true), // v5→v6
            OrientedEdge::new(e6.clone(), true), // v6→v8
            OrientedEdge::new(e7.clone(), true), // v8→v7
            OrientedEdge::new(e8.clone(), true), // v7→v5
        ])
        .unwrap()
        .build_loop(2)
        .unwrap();

        // 上面 (z=1)
        let top_loop = Wire::new(vec![
            OrientedEdge::new(e10.clone(), true),  // v2→v6
            OrientedEdge::new(e6.clone(), true),   // v6→v8
            OrientedEdge::new(e12.clone(), false), // v8→v4
            OrientedEdge::new(e2.clone(), false),  // v4→v2
        ])
        .unwrap()
        .build_loop(3)
        .unwrap();

        // 底面 (z=0)
        let bottom_loop = Wire::new(vec![
            OrientedEdge::new(e4.clone(), false), // v1→v3 逆
            OrientedEdge::new(e11.clone(), true), // v3→v7
            OrientedEdge::new(e8.clone(), true),  // v7→v5
            OrientedEdge::new(e9.clone(), false), // v5→v1 逆
        ])
        .unwrap()
        .build_loop(4)
        .unwrap();

        // 前面 (y=0)
        let front_loop = Wire::new(vec![
            OrientedEdge::new(e9.clone(), true),   // v1→v5
            OrientedEdge::new(e5.clone(), true),   // v5→v6
            OrientedEdge::new(e10.clone(), false), // v6→v2 逆
            OrientedEdge::new(e1.clone(), false),  // v2→v1 逆
        ])
        .unwrap()
        .build_loop(5)
        .unwrap();

        // 背面 (y=1)
        let back_loop = Wire::new(vec![
            OrientedEdge::new(e3.clone(), false),  // v3→v4 逆
            OrientedEdge::new(e12.clone(), true),  // v4→v8
            OrientedEdge::new(e7.clone(), true),   // v8→v7
            OrientedEdge::new(e11.clone(), false), // v7→v3 逆
        ])
        .unwrap()
        .build_loop(6)
        .unwrap();

        // ────────────── 各面の PlaneSurface ──────────────
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

        // ────────────── Face を生成 ──────────────
        let f_left = Face::new(1, left_loop, vec![], left_surf).unwrap();
        let f_right = Face::new(2, right_loop, vec![], right_surf).unwrap();
        let f_top = Face::new(3, top_loop, vec![], top_surf).unwrap();
        let f_bottom = Face::new(4, bottom_loop, vec![], bottom_surf).unwrap();
        let f_front = Face::new(5, front_loop, vec![], front_surf).unwrap();
        let f_back = Face::new(6, back_loop, vec![], back_surf).unwrap();

        // ────────────── 6 面で 1 シェル ──────────────
        let shell = Shell::new(1, vec![f_left, f_right, f_top, f_bottom, f_front, f_back])
            .expect("cube shell should be manifold");

        assert_eq!(shell.faces().len(), 6);
    }
}
