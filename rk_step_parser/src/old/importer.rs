//! cube.step 専用 Importer

use std::collections::HashMap;

use rk_cad::{
    geo::{AnySurface, LineCurve, PlaneSurface},
    topo::{Edge, Face, Loop, Model, Shell, Solid, Vertex, Wire},
    TopologyError,
};
use rk_calc::Vector3;

use crate::attr::Attr;
use crate::builder::Graph;

/* 公開 API ───────────────────────────────────────── */
pub fn import_cube(graph: &Graph) -> Result<Model, TopologyError> {
    /* ── 1. 低次ジオメトリをマップに登録 ───────────────────── */

    /* 1-A  Point3 (=Vector3) */
    let mut cart: HashMap<usize, Vector3> = HashMap::new();
    for n in graph.values().filter(|n| n.kind == "CARTESIAN_POINT") {
        if let Some(Attr::List(coords)) = n.attrs.borrow().get(1) {
            let xyz = coords_as_vec(coords);
            if xyz.len() == 3 {
                cart.insert(n.id, Vector3::new(xyz[0], xyz[1], xyz[2]));
            }
        }
    }

    /* 1-B  Direction */
    let mut dirs: HashMap<usize, Vector3> = HashMap::new();
    for n in graph.values().filter(|n| n.kind == "DIRECTION") {
        if let Some(Attr::List(vals)) = n.attrs.borrow().get(1) {
            let v = coords_as_vec(vals);
            if v.len() == 3 {
                dirs.insert(n.id, Vector3::new(v[0], v[1], v[2]));
            }
        }
    }

    /* 2. Vertex(id, point) */
    let mut verts: HashMap<usize, Vertex> = HashMap::new();
    for n in graph.values().filter(|n| n.kind == "VERTEX_POINT") {
        if let Some(Attr::Ref(pw)) = n.attrs.borrow().get(1) {
            if let Some(p_node) = pw.upgrade() {
                if let Some(p) = cart.get(&p_node.id) {
                    verts.insert(n.id, Vertex::new(n.id, *p));
                }
            }
        }
    }

    /* 3. Edge(id, v1, v2, LineCurve) */
    let mut edges: HashMap<usize, Edge> = HashMap::new();
    for n in graph.values().filter(|n| n.kind == "EDGE_CURVE") {
        let a = ref_vertex(&n.attrs.borrow()[1], &verts);
        let b = ref_vertex(&n.attrs.borrow()[2], &verts);

        if let (Some(va), Some(vb)) = (a, b) {
            let curve = LineCurve::new(va.point(), vb.point());
            edges.insert(n.id, Edge::new(n.id, va, vb, curve)?);
        }
    }

    /* 4. PlaneSurface(id) */
    let mut planes: HashMap<usize, PlaneSurface> = HashMap::new();
    for plane_node in graph.values().filter(|n| n.kind == "PLANE") {
        // attr[1] -> Ref(axis2_placement)
        if let Some(Attr::Ref(pl_w)) = plane_node.attrs.borrow().get(1) {
            if let Some(ax_node) = pl_w.upgrade() {
                // AXIS2_PLACEMENT_3D('', loc, axis, ref_dir)
                let loc_pt = ref_point(&ax_node.attrs.borrow()[1], &cart);
                let axis = ref_dir(&ax_node.attrs.borrow()[2], &dirs)
                    .unwrap_or(Vector3::new(0.0, 0.0, 1.0));

                if let Some(pt) = loc_pt {
                    planes.insert(plane_node.id, PlaneSurface::from_point_normal(pt, axis));
                }
            }
        }
    }

    /* 5. Loop(id, edges, is_outer) */
    let mut loops: HashMap<usize, Loop> = HashMap::new();

    for loop_node in graph.values().filter(|n| n.kind == "EDGE_LOOP") {
        // attr[1] が "( #20, #30, ... )" のリスト
        let attrs_borrowed = loop_node.attrs.borrow();
        let oe_attrs = match attrs_borrowed.get(1) {
            Some(Attr::List(list)) => list,
            _ => continue, // 構文不正なら飛ばす
        };

        let mut oes = Vec::new();

        for a in oe_attrs {
            if let Attr::Ref(oe_w) = a {
                if let Some(oe_node) = oe_w.upgrade() {
                    // ORIENTED_EDGE('',*,*,edge_ref,sense)
                    let forward = match oe_node.attrs.borrow().get(4) {
                        Some(Attr::Scalar(s)) => s.trim() == ".T.",
                        _ => true, // デフォルト正方向
                    };
                    if let Some(Attr::Ref(edge_w)) = oe_node.attrs.borrow().get(3) {
                        if let Some(e_node) = edge_w.upgrade() {
                            if let Some(e) = edges.get(&e_node.id) {
                                oes.push(e.clone().to_oriented_edge(forward));
                            }
                        }
                    }
                }
            }
        }

        // 隣接チェック → ループ生成
        if !oes.is_empty() {
            let wire = Wire::new(oes)?; // 隣接性
            let lp = wire.build_loop(loop_node.id)?; // 閉塞性
            loops.insert(loop_node.id, lp);
        }
    }

    /* 6. Face(id, outer_loop, [], plane) */
    let mut faces: HashMap<usize, Face> = HashMap::new();
    for f_node in graph.values().filter(|n| n.kind == "ADVANCED_FACE") {
        /* outer_loop_ref の取得を 2 段参照に変更 */
        let attrs_borrowed = f_node.attrs.borrow();
        let face_bounds = match attrs_borrowed.get(1) {
            Some(Attr::List(list)) => list,
            _ => continue,
        };

        if let Some(Attr::Ref(fb_w)) = face_bounds.first() {
            if let Some(fb_node) = fb_w.upgrade() {
                // FACE_BOUND('', loop_ref, .T./.F.)
                if let Some(Attr::Ref(loop_w)) = fb_node.attrs.borrow().get(1) {
                    if let Some(loop_node) = loop_w.upgrade() {
                        if let Some(outer) = loops.get(&loop_node.id) {
                            /* surface 取得は以前のまま */
                            if let Some(Attr::Ref(surf_w)) = f_node.attrs.borrow().get(2) {
                                if let Some(plane_node) = surf_w.upgrade() {
                                    if let Some(plane) = planes.get(&plane_node.id) {
                                        faces.insert(
                                            f_node.id,
                                            Face::new(
                                                f_node.id,
                                                outer.clone(),
                                                vec![],
                                                AnySurface::Plane(plane.clone()),
                                            )?,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /* 7. Shell & Solid */
    let mut shells = HashMap::new();
    for s_node in graph.values().filter(|n| n.kind == "CLOSED_SHELL") {
        if let Some(Attr::List(face_refs)) = s_node.attrs.borrow().get(1) {
            let mut s_faces = Vec::new();
            for fr in face_refs {
                if let Attr::Ref(w) = fr {
                    if let Some(f_node) = w.upgrade() {
                        if let Some(f) = faces.get(&f_node.id) {
                            s_faces.push(f.clone());
                        }
                    }
                }
            }
            if !s_faces.is_empty() {
                shells.insert(s_node.id, Shell::new(s_node.id, s_faces)?);
            }
        }
    }

    let mut solids = HashMap::new();
    for b_node in graph.values().filter(|n| n.kind == "MANIFOLD_SOLID_BREP") {
        if let Some(Attr::Ref(sw)) = b_node.attrs.borrow().get(1) {
            if let Some(sh_node) = sw.upgrade() {
                if let Some(sh) = shells.get(&sh_node.id) {
                    solids.insert(b_node.id, Solid::new(b_node.id, sh.clone(), vec![])?);
                }
            }
        }
    }

    /* 8. Modelへ詰める */
    let mut model = Model::new();
    for (_, v) in verts {
        model.add_vertex(v)?;
    }
    for (_, e) in edges {
        model.add_edge(e)?;
    }
    for (_, f) in faces {
        model.add_face(f)?;
    }
    for (_, so) in solids {
        model.add_solid(so)?;
    }
    Ok(model)
}

/* ── ヘルパ ───────────────────────────────────────── */

fn coords_as_vec(list: &[Attr]) -> Vec<f64> {
    list.iter()
        .filter_map(|a| {
            if let Attr::Scalar(s) = a {
                s.trim_end_matches('.').parse::<f64>().ok()
            } else {
                None
            }
        })
        .collect()
}

fn ref_point(attr: &Attr, cart: &HashMap<usize, Vector3>) -> Option<Vector3> {
    if let Attr::Ref(w) = attr {
        w.upgrade().and_then(|n| cart.get(&n.id)).copied()
    } else {
        None
    }
}

fn ref_dir(attr: &Attr, dirs: &HashMap<usize, Vector3>) -> Option<Vector3> {
    if let Attr::Ref(w) = attr {
        w.upgrade().and_then(|n| dirs.get(&n.id)).copied()
    } else {
        None
    }
}

fn ref_vertex<'a>(attr: &Attr, verts: &'a HashMap<usize, Vertex>) -> Option<&'a Vertex> {
    if let Attr::Ref(w) = attr {
        w.upgrade().and_then(|n| verts.get(&n.id))
    } else {
        None
    }
}
