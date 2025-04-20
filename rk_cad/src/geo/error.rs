#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeometryError {
    /// 法線と参照方向（u_axis）がほぼ平行だった
    CollinearAxes,
}
