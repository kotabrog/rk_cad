use rk_calc::Point3;

/// BRep の最小単位：頂点
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    /// 一意に振られた ID
    pub id: usize,
    /// 空間上の位置
    pub point: Point3,
}

impl Vertex {
    /// 新しい頂点を生成
    pub fn new(id: usize, point: Point3) -> Self {
        Vertex { id, point }
    }

    /// 点の座標を更新（必要なら）
    pub fn set_point(&mut self, p: Point3) {
        self.point = p;
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
        assert_eq!(v.id, 1);
        assert_eq!(v.point.x, 1.0);
        assert_eq!(v.point.y, 2.0);
        assert_eq!(v.point.z, 3.0);
    }

    #[test]
    fn vertex_set_point() {
        let mut v = Vertex::new(1, Point3::new(1.0, 2.0, 3.0));
        v.set_point(Point3::new(4.0, 5.0, 6.0));
        assert_eq!(v.point.x, 4.0);
        assert_eq!(v.point.y, 5.0);
        assert_eq!(v.point.z, 6.0);
    }
}
