use ariadne::Report;

pub use thiserror;

pub trait Reportable {
    fn report(&self) -> Option<Vec<Report>> {
        None
    }
}
