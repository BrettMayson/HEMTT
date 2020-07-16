#[macro_use]
extern crate log;

mod cache;
mod tmp;

pub use cache::FileCache;
pub use tmp::Temporary;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
