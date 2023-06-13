use std::{mem::MaybeUninit, sync::mpsc::Sender};

use hemtt_arma::messages::fromarma::Message;

pub struct Conn();

static mut SINGLETON: MaybeUninit<Sender<Message>> = MaybeUninit::uninit();
static mut INIT: bool = false;

impl Conn {
    /// Gets a reference to the sender
    ///
    /// # Panics
    ///
    /// Panics if the sender has not been set
    pub fn get() -> Sender<Message> {
        unsafe { SINGLETON.assume_init_ref().clone() }
    }

    /// Store the sender
    pub fn set(sender: Sender<Message>) {
        unsafe {
            SINGLETON = MaybeUninit::new(sender);
            INIT = true;
        }
    }
}
