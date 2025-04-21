#[derive(Debug)]
pub enum TopologyError {
    /// Edge の両端に同じ頂点 ID が指定された
    EdgeEndpointsEqual,
    /// 隣接しないエッジを push / new しようとした
    EdgesNotContiguous,
    /// Wire が閉じていなかった
    WireNotClosed,
    /// vertex が面上にない
    VertexNotOnSurface(usize),
    /// Shell のエッジがちょうど 2 回現れなかった (edge_id, count)
    ShellNotManifoldEdge(usize, usize),
}
