use super::Entry;

#[derive(Debug, Clone, PartialEq)]
/// An array of entries
pub struct Array {
    /// Is the array expanding a previously defined array
    ///
    /// ```cpp
    /// my_array[] += {1,2,3};
    /// ```
    pub expand: bool,
    /// The elements of the array
    pub elements: Vec<Entry>,
}
