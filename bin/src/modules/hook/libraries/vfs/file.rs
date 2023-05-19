use std::{cell::RefCell, io::Write, rc::Rc};

use rhai::plugin::{
    export_module, mem, Dynamic, FnAccess, FnNamespace, ImmutableString, Module, NativeCallContext,
    PluginFunction, RhaiResult, TypeId,
};
use vfs::SeekAndRead;

#[derive(Clone)]
pub struct ReadFile(Rc<RefCell<Box<dyn SeekAndRead + Send>>>);

#[derive(Clone)]
pub struct WriteFile(Rc<RefCell<Box<dyn Write + Send>>>);

#[export_module]
pub mod file_functions {
    use std::{cell::RefCell, rc::Rc};

    use rhai::EvalAltResult;
    use vfs::VfsPath;

    #[rhai_fn(global, return_raw)]
    pub fn open_file(path: &mut VfsPath) -> Result<ReadFile, Box<EvalAltResult>> {
        path.open_file()
            .map(|f| ReadFile(Rc::new(RefCell::new(f))))
            .map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, return_raw)]
    pub fn create_file(path: &mut VfsPath) -> Result<WriteFile, Box<EvalAltResult>> {
        path.create_file()
            .map(|f| WriteFile(Rc::new(RefCell::new(f))))
            .map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, return_raw)]
    pub fn remove_file(path: &mut VfsPath) -> Result<(), Box<EvalAltResult>> {
        path.remove_file().map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, return_raw)]
    pub fn read(file: &mut ReadFile) -> Result<String, Box<EvalAltResult>> {
        let mut buf = String::new();
        file.0
            .borrow_mut()
            .read_to_string(&mut buf)
            .map(|_| buf)
            .map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, return_raw)]
    pub fn write(file: &mut WriteFile, data: &str) -> Result<(), Box<EvalAltResult>> {
        file.0
            .borrow_mut()
            .write_all(data.as_bytes())
            .map_err(|e| e.to_string().into())
    }
}
