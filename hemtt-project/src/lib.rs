#[macro_use]
extern crate log;

mod defaults;
mod project;
mod template;
pub mod templates;

pub use project::Project;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
