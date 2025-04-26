//! RawEntity → Graph<Node> ＋ 参照解決
//! --------------------------------------
//! 1. build_graph   : 文字列引数を Attr::RefId で保持したまま Node 化
//! 2. resolve_refs  : Attr::RefId → Attr::Ref(Weak<Node>) へ一括変換
//! --------------------------------------

use std::collections::HashMap;
use std::rc::Rc;

use crate::attr::{Attr, Node};
use crate::raw_entity::RawEntity;

pub type Graph = HashMap<usize, Rc<Node>>;

// ───────────────────────────────
// 1) #id 行 → Node グラフ
// ───────────────────────────────
pub fn build_graph(raw: &[RawEntity]) -> Graph {
    let mut g = Graph::with_capacity(raw.len());
    for ent in raw {
        let attrs = Attr::parse_list(&ent.params);
        let node  = Rc::new(Node {
            id:   ent.id,
            kind: ent.keyword.clone(),
            attrs: std::cell::RefCell::new(attrs),
        });
        g.insert(ent.id, node);
    }
    g
}

// ───────────────────────────────
// 2) Attr::RefId を Weak<Node> へ張り替え
// ───────────────────────────────
pub fn resolve_refs(graph: &Graph) {
    for node in graph.values() {
        let mut list = node.attrs.borrow_mut();
        resolve_list(&mut list, graph);
    }
}

fn resolve_list(list: &mut Vec<Attr>, graph: &Graph) {
    for attr in list.iter_mut() {
        match attr {
            Attr::List(inner) => resolve_list(inner, graph),
            Attr::RefId(id) => {
                if let Some(target) = graph.get(id) {
                    *attr = Attr::Ref(Rc::downgrade(target));
                }
            }
            _ => {}
        }
    }
}
