use crate::BISignError;

mod keygen;
pub use keygen::Keygen;

mod sign;
pub use sign::Sign;

mod verify;
pub use verify::Verify;

pub trait Command {
    // (name, description)
    fn register(&self) -> clap::App;
    fn run(&self, _args: &clap::ArgMatches) -> Result<(), BISignError> {
        unimplemented!();
    }
}
