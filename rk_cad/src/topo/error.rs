use thiserror::Error;

/// B-rep トポロジ操作で発生するエラー
#[derive(Debug, Error)]
pub enum TopologyError {
    /// Edge の両端に同じ頂点 ID が指定された
    #[error("edge endpoints are identical")]
    EdgeEndpointsEqual,

    /// 隣接しないエッジを push / new しようとした
    #[error("edges are not contiguous")]
    EdgesNotContiguous,

    /// Wire が閉じていなかった
    #[error("wire is not closed")]
    WireNotClosed,

    /// vertex が面上にない
    #[error("vertex #{0} is not on surface")]
    VertexNotOnSurface(usize),

    /// Shell のエッジがちょうど 2 回現れなかった
    #[error("edge #{0} appears {1} times in shell; manifold violation")]
    ShellNotManifoldEdge(usize, usize),

    /// 内殻の ID が外殻と同じだった
    #[error("inner shell id #{0} is identical to outer shell")]
    InnerShellSameAsOuter(usize),

    /// ID が重複していた
    #[error("{0} id #{1} duplicated")]
    DuplicateId(&'static str, usize),
}
