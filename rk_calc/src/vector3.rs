use super::CalcError;
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

    /// magnitude が 0 の場合にエラーを返す正規化
    pub fn normalize_checked(&self) -> Result<Self, CalcError> {
        const EPS: f64 = 1e-12;
        let mag = self.magnitude();
        if mag < EPS {
            Err(CalcError::ZeroVectorNormalization)
        } else {
            Ok(Vector3::new(self.x / mag, self.y / mag, self.z / mag))
        }
    }

    /// 他のベクトルとの距離（点としての距離計算）
    pub fn distance(self, other: &Self) -> f64 {
        (self - *other).magnitude()
    }

    /// ベクトルの外積
    pub fn cross(self, other: &Self) -> Vector3 {
        Vector3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    /// このベクトルを `axis` 上に射影したベクトルを返す
    /// this·axis /(axis·axis) * axis
    ///
    /// # Errors
    /// - `CalcError::AxisTooSmall`: `axis` がほぼ零ベクトルで射影できない場合
    pub fn project_onto(self, axis: &Vector3) -> Result<Vector3, CalcError> {
        const EPS: f64 = 1e-12;
        let denom = axis.dot(axis);
        if denom.abs() < EPS {
            Err(CalcError::AxisTooSmall)
        } else {
            Ok(*axis * (self.dot(axis) / denom))
        }
    }

    /// Gram–Schmidt で `axis` と直交する単位ベクトル成分を返す
    ///
    /// # Errors
    /// - `CalcError::AxisTooSmall`: 入力軸がほぼ零ベクトルで射影できない場合  
    /// - `CalcError::NoOrthogonalComponent`: 直交成分がほぼ零ベクトルで正規化できない場合
    pub fn orthonormal_component(self, axis: &Vector3) -> Result<Vector3, CalcError> {
        let proj = self.project_onto(axis)?;
        let ortho = self - proj;
        let mag = ortho.magnitude();
        const EPS: f64 = 1e-6;
        if mag < EPS {
            Err(CalcError::NoOrthogonalComponent)
        } else {
            Ok(ortho * (1.0 / mag))
        }
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
    fn vector3_normalize_checked() {
        let vector = Vector3::new(3.0, 4.0, 0.0);
        let normalized = vector.normalize_checked().unwrap();
        assert_eq!(normalized.magnitude(), 1.0); // 正規化後の大きさは 1
        assert_eq!(normalized.x, 0.6);
        assert_eq!(normalized.y, 0.8);
        assert_eq!(normalized.z, 0.0);

        let zero_vector = Vector3::new(0.0, 0.0, 0.0);
        let err = zero_vector.normalize_checked();
        assert!(matches!(err, Err(CalcError::ZeroVectorNormalization)));
    }

    #[test]
    fn vector3_distance() {
        let vector1 = Vector3::new(1.0, 2.0, 3.0);
        let vector2 = Vector3::new(4.0, 5.0, 6.0);
        let distance = vector1.distance(&vector2);
        assert_eq!(distance, 5.196152422706632); // sqrt((4-1)^2 + (5-2)^2 + (6-3)^2) = sqrt(9 + 9 + 9) = sqrt(27)
    }

    #[test]
    fn vector3_cross() {
        let vector1 = Vector3::new(1.0, 2.0, 3.0);
        let vector2 = Vector3::new(4.0, 5.0, 6.0);
        let cross_product = vector1.cross(&vector2);
        assert_eq!(cross_product.x, -3.0);
        assert_eq!(cross_product.y, 6.0);
        assert_eq!(cross_product.z, -3.0);
    }

    #[test]
    fn vector3_cross_easy() {
        let vector1 = Vector3::new(1.0, 0.0, 0.0);
        let vector2 = Vector3::new(0.0, 1.0, 0.0);
        let cross_product = vector1.cross(&vector2);
        assert_eq!(cross_product.x, 0.0);
        assert_eq!(cross_product.y, 0.0);
        assert_eq!(cross_product.z, 1.0);

        let cross_product = vector2.cross(&vector1);
        assert_eq!(cross_product.x, 0.0);
        assert_eq!(cross_product.y, 0.0);
        assert_eq!(cross_product.z, -1.0);
    }

    #[test]
    fn vector3_project_onto() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        let axis = Vector3::new(0.0, 1.0, 0.0);
        let projected = vector.project_onto(&axis).unwrap();
        assert_eq!(projected.x, 0.0);
        assert_eq!(projected.y, 2.0);
        assert_eq!(projected.z, 0.0);
    }

    #[test]
    fn vector3_project_onto_zero_axis() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        let axis = Vector3::new(0.0, 0.0, 0.0);
        let result = vector.project_onto(&axis);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CalcError::AxisTooSmall);
    }

    #[test]
    fn vector3_orthonormal_component() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        let axis = Vector3::new(0.0, 1.0, 0.0);
        let orthogonal = vector.orthonormal_component(&axis).unwrap();
        assert!((orthogonal.magnitude() - 1.0).abs() < 1e-6); // 大きさが 1 であることを確認
        assert!(orthogonal.dot(&axis).abs() < 1e-6); // 直交していることを確認
    }

    #[test]
    fn vector3_orthonormal_component_zero_axis() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        let axis = Vector3::new(0.0, 0.0, 0.0);
        let result = vector.orthonormal_component(&axis);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CalcError::AxisTooSmall);
    }

    #[test]
    fn vector3_add() {
        let vector1 = Vector3::new(1.0, 2.0, 3.0);
        let vector2 = Vector3::new(4.0, 5.0, 6.0);
        let result = vector1 + vector2;
        assert_eq!(result.x, 5.0);
        assert_eq!(result.y, 7.0);
        assert_eq!(result.z, 9.0);
    }

    #[test]
    fn vector3_sub() {
        let vector1 = Vector3::new(4.0, 5.0, 6.0);
        let vector2 = Vector3::new(1.0, 2.0, 3.0);
        let result = vector1 - vector2;
        assert_eq!(result.x, 3.0);
        assert_eq!(result.y, 3.0);
        assert_eq!(result.z, 3.0);
    }

    #[test]
    fn vector3_mul() {
        let vector = Vector3::new(1.0, 2.0, 3.0);
        let scalar = 2.0;
        let result = vector * scalar;
        assert_eq!(result.x, 2.0);
        assert_eq!(result.y, 4.0);
        assert_eq!(result.z, 6.0);
    }
}
