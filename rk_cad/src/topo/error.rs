#[derive(Debug)]
pub enum TopoError {
    /// Edge の両端に同じ頂点 ID が指定された
    EdgeEndpointsEqual,
    /// 隣接しないエッジを push / new しようとした
    EdgesNotContiguous,
    /// Wire が閉じていなかった
    WireNotClosed,
}
