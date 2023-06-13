use hemtt_arma::messages::{fromarma, toarma};

pub trait Action {
    fn missions(&self) -> Vec<(String, String)>;
    fn incoming(&self, msg: fromarma::Message) -> Vec<toarma::Message>;
    fn outgoing(&self) -> Vec<toarma::Message> {
        Vec::new()
    }
}
