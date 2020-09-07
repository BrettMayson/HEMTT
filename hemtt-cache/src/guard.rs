use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use crate::{FileCache, Temporary};

pub struct FileCacheGuard<'a> {
    pub(crate) inner: &'a FileCache,
    pub(crate) data: &'a mut Temporary,
    pub(crate) path: PathBuf,
}

impl<'a> Deref for FileCacheGuard<'a> {
    type Target = Temporary;
    fn deref(&self) -> &Temporary {
        &*self.data
    }
}

impl<'a> DerefMut for FileCacheGuard<'a> {
    fn deref_mut(&mut self) -> &mut Temporary {
        &mut *self.data
    }
}

impl<'a> Drop for FileCacheGuard<'a> {
    fn drop(&mut self) {
        self.inner.unlock(&self.path);
    }
}
