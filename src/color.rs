/// 闪烁覆盖层使用的 RGB 颜色。
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FlashColor {
    /// 红色通道，范围为 `0..=255`。
    pub red: u8,
    /// 绿色通道，范围为 `0..=255`。
    pub green: u8,
    /// 蓝色通道，范围为 `0..=255`。
    pub blue: u8,
}
