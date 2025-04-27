//! Model → RawEntity[] → StepFile
//! -----------------------------------------------------------
//! 生成フロー
//!   1)   CARTESIAN_POINT / VERTEX_POINT
//!   2)   DIRECTION / LINE / EDGE_CURVE / ORIENTED_EDGE
//!   3)   EDGE_LOOP / FACE_BOUND
//!   4)   AXIS2_PLACEMENT_3D / PLANE / ADVANCED_FACE
//!   5)   CLOSED_SHELL / MANIFOLD_SOLID_BREP
//!   6)   GEOMETRIC_REPRESENTATION_CONTEXT
//!   7)   ADVANCED_BREP_SHAPE_REPRESENTATION ツリー
//! -----------------------------------------------------------

use rk_cad::{AnySurface, Loop, Model};
use rk_calc::Vector3;
use std::collections::HashMap;

use crate::raw_entity::RawEntity;
use crate::step_file::StepFile;

/* 公開関数 ───────────────────────────────────────── */
pub fn export_model(model: &Model) -> StepFile {
    let mut next_id = 1_usize;
    let mut entities: Vec<RawEntity> = Vec::new();
    let mut id_map = HashMap::new(); // topo id -> STEP id

    /* 1. CARTESIAN_POINT & VERTEX_POINT */
    for v in model.vertices() {
        let cp_id = next(&mut next_id);
        entities.push(cartesian_point(cp_id, v.point()));

        let vp_id = next(&mut next_id);
        entities.push(vertex_point(vp_id, cp_id));

        id_map.insert(("VERTEX", v.id()), vp_id);
    }

    /* 2. EDGE_CURVE + ORIENTED_EDGE (.T. forward) */
    for e in model.edges() {
        let v1_vp = id_map[&("VERTEX", e.v1().id())];
        let v2_vp = id_map[&("VERTEX", e.v2().id())];

        // LINE
        let p_id = next(&mut next_id);
        entities.push(cartesian_point(p_id, e.v1().point()));

        let dir = (e.v2().point() - e.v1().point()).normalize();
        let dir_id = next(&mut next_id);
        entities.push(direction_entity(dir_id, dir));

        let vec_id = next(&mut next_id);
        entities.push(vector_entity(vec_id, dir_id));

        let line_id = next(&mut next_id);
        entities.push(line(line_id, p_id, vec_id));

        // EDGE_CURVE / ORIENTED_EDGE
        let edge_id = next(&mut next_id);
        entities.push(edge_curve(edge_id, v1_vp, v2_vp, line_id));

        id_map.insert(("EDGE", e.id()), edge_id);
    }

    /* 3. EDGE_LOOP & FACE_BOUND */
    for lp in model.loops() {
        let mut oe_ids = Vec::new();

        for oe in lp.edges() {
            let edge_id = id_map[&("EDGE", oe.edge.id())];
            let oe_id = next(&mut next_id);
            entities.push(oriented_edge(oe_id, edge_id, oe.forward)); // ← 向きを反映
            oe_ids.push(oe_id);
        }

        let loop_id = next(&mut next_id);
        entities.push(edge_loop(loop_id, &oe_ids));
        id_map.insert(("LOOP", lp.id()), loop_id);
    }

    /* 4. PLANE / AXIS2_PLACEMENT_3D / ADVANCED_FACE */
    for f in model.faces() {
        let surf_ref = f.surface();
        let plane = match &*surf_ref {
            AnySurface::Plane(p) => p.clone(),
        };

        let (axis_id, a2p_ents) =
            axis2_placement(next_id, plane.origin, plane.normal, plane.u_axis);
        next_id += 4;
        entities.extend(a2p_ents);

        let plane_id = next(&mut next_id);
        entities.push(plane_surface(plane_id, axis_id));

        let loop_id = id_map[&("LOOP", f.outer().id())];

        let same_sense = {
            // ループ法線と plane.normal の符号で判定するユーティリティ関数
            calc_same_sense(&f.outer(), plane.normal)
        };

        let fb_id = next(&mut next_id);
        entities.push(face_bound(fb_id, loop_id, same_sense));
        id_map.insert(("FBOUND", f.outer().id()), fb_id);

        let af_id = next(&mut next_id);
        entities.push(advanced_face(af_id, fb_id, plane_id, false));
        id_map.insert(("FACE", f.id()), af_id);
    }

    /* 5. CLOSED_SHELL & MANIFOLD_SOLID_BREP */
    for sh in model.iter_shells() {
        let shell_faces = sh
            .faces()
            .iter()
            .map(|f| id_map[&("FACE", f.id())])
            .collect::<Vec<_>>();
        let sh_id = next(&mut next_id);
        entities.push(closed_shell(sh_id, &shell_faces));
        id_map.insert(("SHELL", sh.id()), sh_id);
    }

    // 本サンプルでは Solid は 1 個と仮定
    let solid_id = {
        let so = model.solids().next().expect("no solid");
        let sh_id = id_map[&("SHELL", so.outer().id())];
        let id = next(&mut next_id);
        entities.push(solid_brep(id, sh_id));
        id
    };

    /* ---------- 6. 3D Context + UNIT + UNCERTAINTY -------------- */
    // #ctx
    let ctx_id = next(&mut next_id);
    // #lenUnit, #planeUnit, #solidUnit
    let len_u = next(&mut next_id);
    let ang_u = next(&mut next_id);
    let sol_u = next(&mut next_id);
    // #uncertain
    let uncer = next(&mut next_id);

    // 6-1 長さ単位 (milli-metre)
    entities.push(RawEntity {
        id: len_u,
        keyword: "( LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.) )".into(),
        params: "".into(),
    });
    // 6-2 角度単位 (radian)
    entities.push(RawEntity {
        id: ang_u,
        keyword: "( NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.) )".into(),
        params: "".into(),
    });
    // 6-3 立体角単位 (steradian)
    entities.push(RawEntity {
        id: sol_u,
        keyword: "( NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT() )".into(),
        params: "".into(),
    });

    entities.push(RawEntity {
        id: uncer,
        keyword: "UNCERTAINTY_MEASURE_WITH_UNIT".into(),
        params: format!(
            "LENGTH_MEASURE(1.E-07),#{},'distance_accuracy_value','confusion accuracy'",
            len_u
        ),
    });

    // 6-4 複合コンテキスト (#ctx)
    entities.push(RawEntity {
        id: ctx_id,
        keyword: "( GEOMETRIC_REPRESENTATION_CONTEXT(3) \
    GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((#{})) \
    GLOBAL_UNIT_ASSIGNED_CONTEXT((#{},{},{})) \
    REPRESENTATION_CONTEXT('Context #1','3D Context with UNIT and UNCERTAINTY') )"
            .into(),
        params: "".into(),
    });

    /* ---------- 7. Shape representation tree --------------- */
    let app_id = next(&mut next_id);
    entities.push(RawEntity {
        id: app_id,
        keyword: "APPLICATION_CONTEXT".into(),
        params: "'core data for automotive mechanical design processes'".into(),
    });

    // 7-2 product_context
    let pc_id = next(&mut next_id);
    entities.push(RawEntity {
        id: pc_id,
        keyword: "PRODUCT_CONTEXT".into(),
        params: format!("'' , #{app_id} , 'mechanical'"),
    });

    // 7-3 product
    let prod_id = next(&mut next_id);
    entities.push(RawEntity {
        id: prod_id,
        keyword: "PRODUCT".into(),
        params: format!("'Part','Part','',(#{})", pc_id),
    });

    // 7-4 product_definition_formation
    let pdf_id = next(&mut next_id);
    entities.push(RawEntity {
        id: pdf_id,
        keyword: "PRODUCT_DEFINITION_FORMATION".into(),
        params: format!("'' , '' , #{}", prod_id),
    });

    // 7-5 product_definition_context
    let pdc_id = next(&mut next_id);
    entities.push(RawEntity {
        id: pdc_id,
        keyword: "PRODUCT_DEFINITION_CONTEXT".into(),
        params: format!("'part definition' , #{} , 'design'", app_id),
    });

    // 7-6 product_definition
    let pd_id = next(&mut next_id);
    entities.push(RawEntity {
        id: pd_id,
        keyword: "PRODUCT_DEFINITION".into(),
        params: format!("'design' , '' , #{} , #{}", pdf_id, pdc_id),
    });

    // 7-7 product_definition_shape
    let pds_id = next(&mut next_id);
    entities.push(RawEntity {
        id: pds_id,
        keyword: "PRODUCT_DEFINITION_SHAPE".into(),
        params: format!("'' , '' , #{}", pd_id),
    });

    // 7-8 advanced_brep_shape_representation
    let absr_id = next(&mut next_id);
    entities.push(RawEntity {
        id: absr_id,
        keyword: "ADVANCED_BREP_SHAPE_REPRESENTATION".into(),
        params: format!("'' , (#{}) , #{}", solid_id, ctx_id),
    });

    // 7-9 shape_definition_representation
    let sdr_id = next(&mut next_id);
    entities.push(RawEntity {
        id: sdr_id,
        keyword: "SHAPE_DEFINITION_REPRESENTATION".into(),
        params: format!("#{} , #{}", pds_id, absr_id),
    });

    // 7-10 product_related_product_category（optional だが FreeCAD がメニューに表示）
    let cat_id = next(&mut next_id);
    entities.push(RawEntity {
        id: cat_id,
        keyword: "PRODUCT_RELATED_PRODUCT_CATEGORY".into(),
        params: format!("'part' , $ , (#{})", prod_id),
    });

    /* ---------- HEADER / TRAILER ------------------------- */
    let header = vec![
        "ISO-10303-21;".into(),
        "HEADER;".into(),
        "FILE_DESCRIPTION(('Exported by rk_step_parser'),'2;1');".into(),
        format!(
            "FILE_NAME('cube','{}',(''),(''),'rk_cad','rk_step_parser','');",
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S")
        ),
        "FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));".into(),
        "ENDSEC;".into(),
    ];
    let trailer = vec!["END-ISO-10303-21;".into()];

    StepFile {
        header,
        entities,
        trailer,
    }
}

/* ── RawEntity 生成ヘルパ ────────────────────────── */
fn next(id: &mut usize) -> usize {
    let n = *id;
    *id += 1;
    n
}

fn cartesian_point(id: usize, p: Vector3) -> RawEntity {
    RawEntity {
        id,
        keyword: "CARTESIAN_POINT".into(),
        params: format!("'' , ({:.6},{:.6},{:.6})", p.x, p.y, p.z),
    }
}

fn vertex_point(id: usize, p_id: usize) -> RawEntity {
    RawEntity {
        id,
        keyword: "VERTEX_POINT".into(),
        params: format!("'' , #{p_id}"),
    }
}

fn vector_entity(id: usize, dir_id: usize) -> RawEntity {
    RawEntity {
        id,
        keyword: "VECTOR".into(),
        params: format!("'' , #{} , 1.0", dir_id), // 長さ 1.0 固定
    }
}

fn line(id: usize, p_id: usize, vec_id: usize) -> RawEntity {
    RawEntity {
        id,
        keyword: "LINE".into(),
        params: format!("'' , #{} , #{}", p_id, vec_id),
    }
}

fn direction_entity(id: usize, d: Vector3) -> RawEntity {
    RawEntity {
        id,
        keyword: "DIRECTION".into(),
        params: format!("'' , ({:.6}, {:.6}, {:.6})", d.x, d.y, d.z),
    }
}

fn edge_curve(id: usize, v1: usize, v2: usize, curve_id: usize) -> RawEntity {
    RawEntity {
        id,
        keyword: "EDGE_CURVE".into(),
        params: format!("'' , #{v1}, #{v2}, #{curve_id}, .T."),
    }
}

fn oriented_edge(id: usize, edge_id: usize, forward: bool) -> RawEntity {
    RawEntity {
        id,
        keyword: "ORIENTED_EDGE".into(),
        params: format!(
            "'' , *, *, #{edge_id}, .{}.",
            if forward { "T" } else { "F" }
        ),
    }
}

fn edge_loop(id: usize, oes: &[usize]) -> RawEntity {
    let list = oes
        .iter()
        .map(|i| format!("#{i}"))
        .collect::<Vec<_>>()
        .join(",");
    RawEntity {
        id,
        keyword: "EDGE_LOOP".into(),
        params: format!("'' , ({list})"),
    }
}

/// ループ法線と平面法線が同向きなら true (= `FACE_BOUND .T.`),
/// 逆向きなら false (= `FACE_BOUND .F.`)
pub fn calc_same_sense(lp: &Loop, plane_normal: Vector3) -> bool {
    // 1) Loop の頂点列を順番どおりに取得
    let mut verts: Vec<Vector3> = Vec::with_capacity(lp.edges().len());

    for oe in lp.edges() {
        // forward = true なら Edge.v1 → v2
        // false なら v2 → v1 と進むので start 点は方向に合わせて選ぶ
        let p = if oe.forward {
            oe.edge.v1().point()
        } else {
            oe.edge.v2().point()
        };
        verts.push(p);
    }

    // 2) Newell 法で多角形平均法線を計算
    let mut n = Vector3::new(0.0, 0.0, 0.0);
    for i in 0..verts.len() {
        let (p, q) = (verts[i], verts[(i + 1) % verts.len()]);
        n.x += (p.y - q.y) * (p.z + q.z);
        n.y += (p.z - q.z) * (p.x + q.x);
        n.z += (p.x - q.x) * (p.y + q.y);
    }
    if n.magnitude() == 0.0 {
        // 退化ループ: とりあえず同向き扱い
        return true;
    }
    let loop_normal = n.normalize();

    // 3) 内積の符号で向きを判定
    loop_normal.dot(&plane_normal) > 0.0
}

fn face_bound(id: usize, loop_id: usize, same: bool) -> RawEntity {
    RawEntity {
        id,
        keyword: "FACE_BOUND".into(),
        params: format!("'' , #{loop_id}, .{}.", if same { "T" } else { "F" }),
    }
}

/// origin = 平面上の点
/// normal = 法線（単位化されている前提）
/// u_axis = normal と直交する単位ベクトル
///
/// 返り値: (axis2_id, 生成した RawEntity Vec)
fn axis2_placement(
    id_base: usize,
    origin: Vector3,
    normal: Vector3,
    u_axis: Vector3,
) -> (usize, Vec<RawEntity>) {
    let mut es = Vec::new();
    es.push(cartesian_point(id_base, origin));
    es.push(direction_entity(id_base + 1, normal));
    es.push(direction_entity(id_base + 2, u_axis));
    let a2p_id = id_base + 3;
    es.push(RawEntity {
        id: a2p_id,
        keyword: "AXIS2_PLACEMENT_3D".into(),
        params: format!("'' , #{} , #{} , #{}", id_base, id_base + 1, id_base + 2),
    });
    (a2p_id, es)
}

fn plane_surface(id: usize, ax_id: usize) -> RawEntity {
    RawEntity {
        id,
        keyword: "PLANE".into(),
        params: format!("'' , #{ax_id}"),
    }
}

fn advanced_face(id: usize, fb_id: usize, surf_id: usize, same: bool) -> RawEntity {
    RawEntity {
        id,
        keyword: "ADVANCED_FACE".into(),
        params: format!(
            "'' , (#{fb_id}), #{surf_id}, .{}.",
            if same { "T" } else { "F" }
        ),
    }
}

fn closed_shell(id: usize, faces: &[usize]) -> RawEntity {
    let list = faces
        .iter()
        .map(|i| format!("#{i}"))
        .collect::<Vec<_>>()
        .join(",");
    RawEntity {
        id,
        keyword: "CLOSED_SHELL".into(),
        params: format!("'' , ({list})"),
    }
}

fn solid_brep(id: usize, shell_id: usize) -> RawEntity {
    RawEntity {
        id,
        keyword: "MANIFOLD_SOLID_BREP".into(),
        params: format!("'' , #{shell_id}"),
    }
}
