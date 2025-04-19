use rk_calc::Point3;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

#[derive(Debug)]
pub struct VertexData {
    /// 一意 ID
    id: usize,
    /// 空間上の座標
    point: Point3,
}

impl VertexData {
    fn new(id: usize, point: Point3) -> Self {
        VertexData { id, point }
    }
}

/// Rc<RefCell<VertexData>> をラップした型
#[derive(Clone, Debug)]
pub struct Vertex(Rc<RefCell<VertexData>>);

impl Vertex {
    /// 新しい Vertex を生成
    pub fn new(id: usize, point: Point3) -> Self {
        Vertex(Rc::new(RefCell::new(VertexData::new(id, point))))
    }

    /// ID を取得
    pub fn id(&self) -> usize {
        self.0.borrow().id
    }

    /// 座標を取得
    pub fn point(&self) -> Point3 {
        self.0.borrow().point
    }

    /// 座標を更新
    pub fn set_point(&self, p: Point3) {
        self.0.borrow_mut().point = p;
    }

    /// 内部データへの不変借用
    pub fn borrow(&self) -> Ref<'_, VertexData> {
        self.0.borrow()
    }

    /// 内部データへの可変借用
    pub fn borrow_mut(&self) -> RefMut<'_, VertexData> {
        self.0.borrow_mut()
    }

    /// 2つの Vertex が同じ ID を持つか比較
    pub fn same_id(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    /// 2つの Vertex が同じ座標を持つか比較
    pub fn same_point(&self, other: &Self) -> bool {
        self.point() == other.point()
    }

    /// 2つの Vertex のユークリッド距離を計算
    pub fn distance(&self, other: &Self) -> f64 {
        self.point().distance(&other.point())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rk_calc::Point3;

    #[test]
    fn vertex_new() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let v = Vertex::new(1, p);
        assert_eq!(v.id(), 1);
        assert_eq!(v.point().x, 1.0);
        assert_eq!(v.point().y, 2.0);
        assert_eq!(v.point().z, 3.0);
    }

    #[test]
    fn vertex_set_point() {
        let v = Vertex::new(1, Point3::new(1.0, 2.0, 3.0));
        v.set_point(Point3::new(4.0, 5.0, 6.0));
        assert_eq!(v.point().x, 4.0);
        assert_eq!(v.point().y, 5.0);
        assert_eq!(v.point().z, 6.0);
    }

    #[test]
    fn vertex_borrow() {
        let v = Vertex::new(1, Point3::new(1.0, 2.0, 3.0));
        let borrowed = v.borrow();
        assert_eq!(borrowed.id, 1);
        assert_eq!(borrowed.point.x, 1.0);
        assert_eq!(borrowed.point.y, 2.0);
        assert_eq!(borrowed.point.z, 3.0);
    }

    #[test]
    fn vertex_borrow_mut() {
        let v = Vertex::new(1, Point3::new(1.0, 2.0, 3.0));
        {
            let mut borrowed = v.borrow_mut();
            borrowed.point = Point3::new(4.0, 5.0, 6.0);
        }
        assert_eq!(v.point().x, 4.0);
        assert_eq!(v.point().y, 5.0);
        assert_eq!(v.point().z, 6.0);
    }

    #[test]
    fn vertex_same_id() {
        let v1 = Vertex::new(1, Point3::new(1.0, 2.0, 3.0));
        let v2 = Vertex::new(1, Point3::new(4.0, 5.0, 6.0));
        assert!(v1.same_id(&v2));
    }

    #[test]
    fn vertex_same_point() {
        let v1 = Vertex::new(1, Point3::new(1.0, 2.0, 3.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 2.0, 3.0));
        assert!(v1.same_point(&v2));
    }

    #[test]
    fn vertex_distance() {
        let v1 = Vertex::new(1, Point3::new(1.0, 2.0, 3.0));
        let v2 = Vertex::new(2, Point3::new(4.0, 5.0, 6.0));
        assert_eq!(v1.distance(&v2), v1.point().distance(&v2.point()));
    }
}
