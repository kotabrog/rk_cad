use std::collections::HashMap;

use super::{Edge, Face, Loop, Shell, Solid, TopologyError, Vertex};

/// B-rep 全要素を格納するトップレベル
#[derive(Debug, Default)]
pub struct Model {
    // プリミティブ
    vertices: HashMap<usize, Vertex>,
    edges: HashMap<usize, Edge>,
    faces: HashMap<usize, Face>,

    // アグリゲート
    solids: HashMap<usize, Solid>, // Shell は Solid 内だけ
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Vertex> {
        self.vertices.values()
    }
    pub fn edges(&self) -> impl Iterator<Item = &Edge> {
        self.edges.values()
    }
    pub fn faces(&self) -> impl Iterator<Item = &Face> {
        self.faces.values()
    }
    pub fn solids(&self) -> impl Iterator<Item = &Solid> {
        self.solids.values()
    }

    pub fn add_vertex(&mut self, v: Vertex) -> Result<(), TopologyError> {
        let id = v.id();
        if self.vertices.insert(id, v).is_some() {
            return Err(TopologyError::DuplicateId("Vertex", id));
        }
        Ok(())
    }

    pub fn add_edge(&mut self, e: Edge) -> Result<(), TopologyError> {
        let id = e.id();
        if self.edges.insert(id, e).is_some() {
            return Err(TopologyError::DuplicateId("Edge", id));
        }
        Ok(())
    }

    pub fn add_face(&mut self, f: Face) -> Result<(), TopologyError> {
        let id = f.id();
        if self.faces.insert(id, f).is_some() {
            return Err(TopologyError::DuplicateId("Face", id));
        }
        Ok(())
    }

    pub fn add_solid(&mut self, so: Solid) -> Result<(), TopologyError> {
        let id = so.id;
        if self.solids.insert(id, so).is_some() {
            return Err(TopologyError::DuplicateId("Solid", id));
        }
        Ok(())
    }

    pub fn vertex(&self, id: usize) -> Option<&Vertex> {
        self.vertices.get(&id)
    }
    pub fn edge(&self, id: usize) -> Option<&Edge> {
        self.edges.get(&id)
    }
    pub fn face(&self, id: usize) -> Option<&Face> {
        self.faces.get(&id)
    }
    pub fn solid(&self, id: usize) -> Option<&Solid> {
        self.solids.get(&id)
    }

    /// モデル内のすべての Loop を *値* で返すイテレータ
    /// （各 Face の outer + inners）
    pub fn loops(&self) -> impl Iterator<Item = Loop> + '_ {
        self.faces.values().flat_map(|f| {
            // 1つの Vec に outer + inners を所有権ごと集める
            let mut all = Vec::with_capacity(1 + f.inners().len());
            all.push(f.outer().clone()); // outer
            all.extend(f.inners().iter().cloned()); // inners
            all.into_iter() // 所有イテレータ
        })
    }

    /// Shell を横断列挙したい場合のヘルパ
    pub fn iter_shells(&self) -> impl Iterator<Item = &Shell> {
        self.solids
            .values()
            .flat_map(|so| std::iter::once(so.outer()).chain(so.inners()))
    }
}

/* ─────────────────── STEP 出力の骨格 ─────────────────── */

impl Model {
    /// （簡易）STEP テキストを生成  
    /// 実際には依存順に more entity を書き出します
    pub fn to_step_string(&self) -> String {
        let mut lines = Vec::<String>::new();
        let mut line_no = 1usize;

        // 1) 頂点を ID 昇順で書く
        let mut verts: Vec<&Vertex> = self.vertices.values().collect();
        verts.sort_by_key(|v| v.id());
        for v in verts {
            lines.push(format!(
                "#{n} = CARTESIAN_POINT('', ({:.6},{:.6},{:.6}));",
                v.point().x,
                v.point().y,
                v.point().z,
                n = line_no
            ));
            line_no += 1;
        }

        // 2) エッジ・フェース … も同様にソート→出力
        //    略

        lines.join("\n")
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

    #[test]
    fn model_with_cube_manual_register() {
        /* ────────────── 1) プリミティブ ────────────── */
        let v = [
            Vertex::new(1, Vector3::new(0.0, 0.0, 0.0)),
            Vertex::new(2, Vector3::new(0.0, 0.0, 1.0)),
            Vertex::new(3, Vector3::new(0.0, 1.0, 0.0)),
            Vertex::new(4, Vector3::new(0.0, 1.0, 1.0)),
            Vertex::new(5, Vector3::new(1.0, 0.0, 0.0)),
            Vertex::new(6, Vector3::new(1.0, 0.0, 1.0)),
            Vertex::new(7, Vector3::new(1.0, 1.0, 0.0)),
            Vertex::new(8, Vector3::new(1.0, 1.0, 1.0)),
        ];
        let e = [
            Edge::new_line(1, &v[0], &v[1]).unwrap(),
            Edge::new_line(2, &v[1], &v[3]).unwrap(),
            Edge::new_line(3, &v[3], &v[2]).unwrap(),
            Edge::new_line(4, &v[2], &v[0]).unwrap(),
            Edge::new_line(5, &v[4], &v[5]).unwrap(),
            Edge::new_line(6, &v[5], &v[7]).unwrap(),
            Edge::new_line(7, &v[7], &v[6]).unwrap(),
            Edge::new_line(8, &v[6], &v[4]).unwrap(),
            Edge::new_line(9, &v[0], &v[4]).unwrap(),
            Edge::new_line(10, &v[1], &v[5]).unwrap(),
            Edge::new_line(11, &v[2], &v[6]).unwrap(),
            Edge::new_line(12, &v[3], &v[7]).unwrap(),
        ];

        let mk_loop = |spec: &[(usize, bool)], id| {
            Wire::new(
                spec.iter()
                    .map(|&(ei, f)| OrientedEdge::new(e[ei - 1].clone(), f))
                    .collect(),
            )
            .unwrap()
            .build_loop(id)
            .unwrap()
        };
        let left = mk_loop(&[(1, true), (2, true), (3, true), (4, true)], 1);
        let right = mk_loop(&[(5, true), (6, true), (7, true), (8, true)], 2);
        let top = mk_loop(&[(10, true), (6, true), (12, false), (2, false)], 3);
        let bottom = mk_loop(&[(4, false), (11, true), (8, true), (9, false)], 4);
        let front = mk_loop(&[(9, true), (5, true), (10, false), (1, false)], 5);
        let back = mk_loop(&[(3, false), (12, true), (7, true), (11, false)], 6);

        fn orthogonal_unit(n: Vector3) -> Vector3 {
            // X に近い向きなら Y 軸を、そうでなければ X 軸を基に直交化
            let ref_dir = if n.x.abs() > 0.9 {
                Vector3::new(0.0, 1.0, 0.0)
            } else {
                Vector3::new(1.0, 0.0, 0.0)
            };
            // Gram–Schmidt で直交単位ベクトルを作る
            ref_dir.orthonormal_component(&n).unwrap()
        }

        let surf = |o: Vector3, n: Vector3| -> AnySurface {
            let u = orthogonal_unit(n);
            PlaneSurface::new(o, n, u).unwrap().into()
        };

        let faces = [
            Face::new(
                1,
                left,
                vec![],
                surf(Vector3::new(0.0, 0.0, 0.0), Vector3::new(-1.0, 0.0, 0.0)),
            )
            .unwrap(),
            Face::new(
                2,
                right,
                vec![],
                surf(Vector3::new(1.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0)),
            )
            .unwrap(),
            Face::new(
                3,
                top,
                vec![],
                surf(Vector3::new(0.0, 0.0, 1.0), Vector3::new(0.0, 0.0, 1.0)),
            )
            .unwrap(),
            Face::new(
                4,
                bottom,
                vec![],
                surf(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, -1.0)),
            )
            .unwrap(),
            Face::new(
                5,
                front,
                vec![],
                surf(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, -1.0, 0.0)),
            )
            .unwrap(),
            Face::new(
                6,
                back,
                vec![],
                surf(Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 1.0, 0.0)),
            )
            .unwrap(),
        ];

        /* ────────────── 2) Model にプリミティブ登録 ────────────── */
        let mut model = Model::new();
        for vtx in &v {
            model.add_vertex(vtx.clone()).unwrap();
        }
        for edg in &e {
            model.add_edge(edg.clone()).unwrap();
        }
        for f in &faces {
            model.add_face(f.clone()).unwrap();
        }

        /* ────────────── 3) Shell / Solid 生成 & 登録 ────────────── */
        let shell = Shell::new(1, faces.to_vec()).unwrap();
        let solid = Solid::new(1, shell, Vec::new()).unwrap();
        model.add_solid(solid).unwrap();

        /* ────────────── 4) 検証 ────────────── */
        assert_eq!(model.vertices().count(), 8);
        assert_eq!(model.edges().count(), 12);
        assert_eq!(model.faces().count(), 6);
        assert_eq!(model.solids().count(), 1);

        // Shell 列挙は 1 つ
        assert_eq!(model.iter_shells().count(), 1);
    }
}
