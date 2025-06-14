/// `rk_calc` 全体で利用するエラー型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    /// 射影対象の軸ベクトルがほぼ零ベクトルで、射影できない
    AxisTooSmall,
    /// Gram–Schmidt 直交化で、直交成分がほぼ零ベクトルになった
    NoOrthogonalComponent,
    /// 正規化しようとしたベクトルがほぼ零ベクトルだった
    ZeroVectorNormalization,
}
