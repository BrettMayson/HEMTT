use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct List<L>(pub L);

impl<L> fmt::Display for List<L>
where
    L: IntoIterator + Clone,
    L::Item: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, item) in self.0.clone().into_iter().enumerate() {
            if i != 0 {
                f.write_str(", ")?;
            }
            fmt::Display::fmt(&item, f)?;
        }

        Ok(())
    }
}
