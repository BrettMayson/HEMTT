use hemtt::Addon;

use super::Context;

pub struct AddonContext<'a, 'b> {
    pub global: &'b Context<'a>,
    pub addon: &'b Addon,
}

pub struct AddonListContext<'a, 'b> {
    pub global: &'b Context<'a>,
    pub addons: &'b mut crate::AddonList,
}
