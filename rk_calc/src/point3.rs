/// 3D 空間上の点・ベクトルを表す型
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point3 {
    /// 新しい Point3 を作成
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Point3 { x, y, z }
    }

    /// 他の点とのユークリッド距離
    pub fn distance(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

// ベクトル同士の加減算、スカラー倍を実装しておくと便利
use std::ops::{Add, Mul, Sub};

impl Add for Point3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Point3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Point3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Point3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Point3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self::Output {
        Point3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point3_new() {
        let p = Point3::new(1.0, 2.0, 3.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
        assert_eq!(p.z, 3.0);
    }

    #[test]
    fn point3_distance() {
        let p1 = Point3::new(1.0, 2.0, 3.0);
        let p2 = Point3::new(4.0, 5.0, 6.0);
        assert_eq!(
            p1.distance(&p2),
            ((4.0f64 - 1.0) * (4.0 - 1.0) + (5.0 - 2.0) * (5.0 - 2.0) + (6.0 - 3.0) * (6.0 - 3.0))
                .sqrt()
        );
    }

    #[test]
    fn point3_add() {
        let p1 = Point3::new(1.0, 2.0, 3.0);
        let p2 = Point3::new(4.0, 5.0, 6.0);
        let p3 = p1 + p2;
        assert_eq!(p3.x, 5.0);
        assert_eq!(p3.y, 7.0);
        assert_eq!(p3.z, 9.0);
    }

    #[test]
    fn point3_sub() {
        let p1 = Point3::new(4.0, 5.0, 6.0);
        let p2 = Point3::new(1.0, 2.0, 3.0);
        let p3 = p1 - p2;
        assert_eq!(p3.x, 3.0);
        assert_eq!(p3.y, 3.0);
        assert_eq!(p3.z, 3.0);
    }

    #[test]
    fn point3_mul() {
        let p = Point3::new(1.0, 2.0, 3.0);
        let p2 = p * 2.0;
        assert_eq!(p2.x, 2.0);
        assert_eq!(p2.y, 4.0);
        assert_eq!(p2.z, 6.0);
    }
}
