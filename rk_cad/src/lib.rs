// rk_cad/src/lib.rs
//! CADライブラリ。ここでは簡単な立方体（Block）の定義と、CADモデル(CadModel)の管理を行います。

pub mod geo;
pub mod topo;

pub use geo::{AnyCurve, Curve, LineCurve};
use rk_calc::Vector3;
pub use topo::{Edge, Face, Loop, OrientedEdge, Vertex, Wire};

/// CADの形状に共通する振る舞いを定義するトレイト。
pub trait Shape {
    /// 形状の名称を返す
    fn name(&self) -> &str;
}

/// 立方体（または直方体）を表す構造体。
#[derive(Debug, Clone)]
pub struct Block {
    /// ブロックの名称
    pub name: String,
    /// ブロックの原点（左下前方の角など）
    pub origin: Vector3,
    /// 各軸方向の寸法
    pub dimensions: Vector3,
}

impl Block {
    /// 新しいBlockを生成する
    pub fn new(name: &str, origin: Vector3, dimensions: Vector3) -> Self {
        Self {
            name: name.to_string(),
            origin,
            dimensions,
        }
    }
}

impl Shape for Block {
    fn name(&self) -> &str {
        &self.name
    }
}

/// CADモデル。ここではBlockを複数保持する簡単なモデルとします。
#[derive(Debug)]
pub struct CadModel {
    pub blocks: Vec<Block>,
}

impl CadModel {
    /// 新規の空のCADモデルを作成する
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    /// モデルにBlockを追加する
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rk_calc::Vector3;

    #[test]
    fn test_add_block() {
        let mut model = CadModel::new();
        let block = Block::new(
            "Block1",
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 2.0, 3.0),
        );
        model.add_block(block.clone());
        assert_eq!(model.blocks.len(), 1);
        assert_eq!(model.blocks[0].name, "Block1");
    }
}
