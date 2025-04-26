use std::collections::HashMap;
use std::rc::Rc;
use super::{Attr, RawEntity};

/// グラフの 1 ノード
#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub kind: String,
    pub attrs: Vec<Attr>,          // 再帰構造
}

pub type Graph = HashMap<usize, Rc<Node>>;

/// エンティティ配列から参照グラフを生成
pub fn build_graph(entities: &[RawEntity]) -> Graph {
    let mut g: Graph = HashMap::with_capacity(entities.len());
    for ent in entities {
        let attrs = Attr::parse_list(&ent.params);
        let node = Rc::new(Node { id: ent.id, kind: ent.keyword.clone(), attrs });
        g.insert(ent.id, node);
    }
    g
}
