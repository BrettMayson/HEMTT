#[derive(Debug, Clone, PartialEq)]
/// A number value
pub enum Number {
    /// A 32-bit integer
    Int32(i32),
    /// A 64-bit integer
    Int64(i64),
    /// A 32-bit floating point number
    Float32(f32),
}
