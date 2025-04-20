use std::ops::{Add, Mul, Sub};

/// 3D ベクトル／点を表す型（名前を Vector3 に変更）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    /// 新しい Vector3 を作成
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Vector3 { x, y, z }
    }

    /// 内積
    pub fn dot(self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 大きさ（ノルム）
    pub fn magnitude(&self) -> f64 {
        self.dot(self).sqrt()
    }

    /// 正規化（大きさを 1 にする）
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        Vector3::new(self.x / mag, self.y / mag, self.z / mag)
    }

    /// 他のベクトルとの距離（点としての距離計算）
    pub fn distance(self, other: &Self) -> f64 {
        (self - *other).magnitude()
    }
}

impl Add for Vector3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Vector3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Sub for Vector3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Mul<f64> for Vector3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector3_new() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(vector.x, 1.0);
        assert_eq!(vector.y, 2.0);
        assert_eq!(vector.z, 3.0);
    }

    #[test]
    fn vector3_dot() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        let other = Vector3::new(4.0, 5.0, 6.0);
        let result = vector.dot(&other);
        assert_eq!(result, 32.0); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    }

    #[test]
    fn vector3_magnitude() {
        let vector = Vector3::new(3.0, 4.0, 0.0);
        let result = vector.magnitude();
        assert_eq!(result, 5.0); // sqrt(3^2 + 4^2) = sqrt(9 + 16) = sqrt(25) = 5
    }

    #[test]
    fn vector3_normalize() {
        let vector = Vector3::new(3.0, 4.0, 0.0);
        let normalized = vector.normalize();
        assert_eq!(normalized.magnitude(), 1.0); // 正規化後の大きさは 1
        assert_eq!(normalized.x, 0.6);
        assert_eq!(normalized.y, 0.8);
        assert_eq!(normalized.z, 0.0);
    }

    #[test]
    fn vector3_distance() {
        let vector1 = Vector3::new(1.0, 2.0, 3.0);
        let vector2 = Vector3::new(4.0, 5.0, 6.0);
        let distance = vector1.distance(&vector2);
        assert_eq!(distance, 5.196152422706632); // sqrt((4-1)^2 + (5-2)^2 + (6-3)^2) = sqrt(9 + 9 + 9) = sqrt(27)
    }
}
