#[derive(Copy, Clone, Debug, Default)]
pub struct LinterOptions {
    class_inheritance_style: InheritanceStyle,
}

#[derive(Copy, Clone, Debug)]
pub enum InheritanceStyle {
    /// The colon is preceded by whitespace
    ///
    ///```cpp
    ///class my_class : parent_class;
    ///```
    Space,
    /// The colon is not preceded by whitespace
    ///
    ///```cpp
    ///class my_class: parent_class;
    ///```
    NoSpace,
}
impl Default for InheritanceStyle {
    fn default() -> Self {
        InheritanceStyle::NoSpace
    }
}
