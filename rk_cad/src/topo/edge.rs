use super::Vertex;
use rk_calc::Point3;

/// Edge は 2つの Vertex を直接保持
#[derive(Debug, Clone)]
pub struct Edge {
    pub id: usize,
    pub v1: Vertex,
    pub v2: Vertex,
}

impl Edge {
    pub fn new(id: usize, v1: &Vertex, v2: &Vertex) -> Self {
        assert!(v1.id() != v2.id(), "Edge endpoints must differ");
        Edge {
            id,
            v1: v1.clone(),
            v2: v2.clone(),
        }
    }

    pub fn length(&self) -> f64 {
        self.v1.distance(&self.v2)
    }

    pub fn translate_endpoints(&self, delta: Point3) {
        self.v1.set_point(self.v1.point() + delta);
        self.v2.set_point(self.v2.point() + delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rk_calc::Point3;

    #[test]
    fn edge_new() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 1.0, 1.0));
        let edge = Edge::new(1, &v1, &v2);
        assert_eq!(edge.id, 1);
        assert_eq!(edge.v1.id(), 1);
        assert_eq!(edge.v2.id(), 2);
    }

    #[test]
    fn edge_length() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(3.0, 4.0, 0.0));
        let edge = Edge::new(1, &v1, &v2);
        assert_eq!(edge.length(), 5.0);
    }

    #[test]
    fn edge_translate_endpoints() {
        let v1 = Vertex::new(1, Point3::new(0.0, 0.0, 0.0));
        let v2 = Vertex::new(2, Point3::new(1.0, 1.0, 1.0));
        let edge = Edge::new(1, &v1, &v2);
        edge.translate_endpoints(Point3::new(1.0, 2.0, 3.0));
        assert_eq!(edge.v1.point().x, 1.0);
        assert_eq!(edge.v1.point().y, 2.0);
        assert_eq!(edge.v1.point().z, 3.0);
        assert_eq!(edge.v2.point().x, 2.0);
        assert_eq!(edge.v2.point().y, 3.0);
        assert_eq!(edge.v2.point().z, 4.0);
    }
}
