use super::GeometryError;
use rk_calc::Vector3;

/// ───────────────────────────────────────────
/// 曲面の共通トレイト
/// ───────────────────────────────────────────
pub trait Surface: std::fmt::Debug + Clone + PartialEq {
    /// パラメータ (u, v) に対応する 3D 座標を返す
    fn position(&self, u: f64, v: f64) -> Vector3;
    /// パラメータ (u, v) に対応する法線ベクトルを返す
    fn normal(&self, u: f64, v: f64) -> Vector3;
    /// この Surface 上に点 p があるか許容誤差 eps で返す
    fn contains_point(&self, p: &Vector3, eps: f64) -> bool;
}

/// ───────────────────────────────────────────
/// 平面曲面
/// ───────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub struct PlaneSurface {
    /// オリジン（原点）位置
    pub origin: Vector3,
    /// 面の法線（単位ベクトル）
    pub normal: Vector3,
    /// U パラメータ軸方向（単位ベクトル）
    pub u_axis: Vector3,
    /// V パラメータ軸方向（単位ベクトル）
    pub v_axis: Vector3,
}

impl PlaneSurface {
    /// origin, 法線, 参照方向 (u_axis) を受け取り、
    /// Vector3::orthonormal_component を使って自動的に直交基底を構築します。
    ///
    /// # Errors
    /// - `SurfaceError::CollinearAxes`:
    ///   `normal` と `u_axis` がほぼ平行で直交化できない場合
    pub fn new(origin: Vector3, normal: Vector3, u_axis: Vector3) -> Result<Self, GeometryError> {
        // 1) 法線を単位化
        let n = normal.normalize();
        // 2) u_axis を n に直交化して単位ベクトル化
        let u = u_axis
            .orthonormal_component(&n)
            .map_err(|_| GeometryError::CollinearAxes)?;
        // 3) 第三の軸は外積で得て正規化
        let v = n.cross(&u).normalize();

        Ok(PlaneSurface {
            origin,
            normal: n,
            u_axis: u,
            v_axis: v,
        })
    }
}

impl Surface for PlaneSurface {
    fn position(&self, u: f64, v: f64) -> Vector3 {
        // origin + u*u_axis + v*v_axis
        self.origin + self.u_axis * u + self.v_axis * v
    }

    fn normal(&self, _u: f64, _v: f64) -> Vector3 {
        // 常に法線ベクトルを返す
        self.normal
    }

    fn contains_point(&self, p: &Vector3, eps: f64) -> bool {
        // p が平面上にあるかどうかを判定
        // p が平面上にあるなら、p - origin と法線の内積は 0 になる
        let d = (*p - self.origin).dot(&self.normal);
        d.abs() <= eps
    }
}

/// ───────────────────────────────────────────
/// 拡張可能な列挙型 AnySurface
/// ───────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum AnySurface {
    Plane(PlaneSurface),
    // Cylinder(CylinderSurface),
    // Nurbs(NurbsSurface),
}

impl From<PlaneSurface> for AnySurface {
    fn from(p: PlaneSurface) -> Self {
        AnySurface::Plane(p)
    }
}

impl Surface for AnySurface {
    fn position(&self, u: f64, v: f64) -> Vector3 {
        match self {
            AnySurface::Plane(p) => p.position(u, v),
            // AnySurface::Cylinder(c) => c.position(u, v),
            // AnySurface::Nurbs(n)    => n.position(u, v),
        }
    }

    fn normal(&self, u: f64, v: f64) -> Vector3 {
        match self {
            AnySurface::Plane(p) => p.normal(u, v),
            // …
        }
    }

    fn contains_point(&self, p: &Vector3, eps: f64) -> bool {
        match self {
            AnySurface::Plane(plane) => plane.contains_point(p, eps),
            // AnySurface::Cylinder(c) => c.contains_point(p, eps),
            // AnySurface::Nurbs(n)    => n.contains_point(p, eps),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rk_calc::Vector3;

    #[test]
    fn plane_surface() {
        let origin = Vector3::new(0.0, 0.0, 0.0);
        let normal = Vector3::new(0.0, 0.0, 1.0);
        let u_axis = Vector3::new(1.0, 0.0, 0.0);

        let plane = PlaneSurface::new(origin, normal, u_axis).unwrap();

        assert_eq!(plane.origin, origin);
        assert_eq!(plane.normal, normal.normalize());
        assert_eq!(plane.u_axis, u_axis.normalize());
        assert_eq!(plane.v_axis, Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(plane.position(1.0, 2.0), Vector3::new(1.0, 2.0, 0.0));
        assert_eq!(plane.normal(1.0, 2.0), normal.normalize());
        assert_eq!(plane.position(0.0, 0.0), origin);
        assert_eq!(plane.position(1.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
        assert_eq!(plane.position(0.0, 1.0), Vector3::new(0.0, 1.0, 0.0));
        assert_eq!(plane.position(1.0, 1.0), Vector3::new(1.0, 1.0, 0.0));
    }

    #[test]
    #[should_panic(expected = "CollinearAxes")]
    fn plane_surface_collinear_axes() {
        let origin = Vector3::new(0.0, 0.0, 0.0);
        let normal = Vector3::new(0.0, 0.0, 1.0);
        let u_axis = Vector3::new(0.0, 0.0, 2.0); // 法線と平行

        PlaneSurface::new(origin, normal, u_axis).unwrap();
    }

    #[test]
    fn plane_surface_contains_point() {
        let origin = Vector3::new(0.0, 0.0, 0.0);
        let normal = Vector3::new(0.0, 0.0, 1.0);
        let u_axis = Vector3::new(1.0, 0.0, 0.0);
        let plane = PlaneSurface::new(origin, normal, u_axis).unwrap();

        let point_on_plane = Vector3::new(1.0, 2.0, 0.0);
        let point_off_plane = Vector3::new(1.0, 2.0, 1.0);

        assert!(plane.contains_point(&point_on_plane, 1e-6));
        assert!(!plane.contains_point(&point_off_plane, 1e-6));
    }
}
