use std::{cell::RefCell, fs::File, io::Write, rc::Rc};

use rhai::plugin::{
    export_module, mem, Dynamic, FnAccess, FnNamespace, ImmutableString, Module, NativeCallContext,
    PluginFunction, RhaiResult, TypeId,
};

#[derive(Clone)]
pub struct ReadFile(Rc<RefCell<File>>);

#[derive(Clone)]
pub struct WriteFile(Rc<RefCell<File>>);

#[export_module]
pub mod file_functions {
    use std::{fs::File, io::Read, path::PathBuf, rc::Rc};

    use rhai::EvalAltResult;

    #[rhai_fn(global, return_raw)]
    pub fn open_file(path: &mut PathBuf) -> Result<ReadFile, Box<EvalAltResult>> {
        File::open(path)
            .map(|f| ReadFile(Rc::new(RefCell::new(f))))
            .map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, return_raw)]
    pub fn create_file(path: &mut PathBuf) -> Result<WriteFile, Box<EvalAltResult>> {
        File::create(path)
            .map(|f| WriteFile(Rc::new(RefCell::new(f))))
            .map_err(|e| e.to_string().into())
    }

    #[rhai_fn(global, return_raw)]
    pub fn remove_file(path: &mut PathBuf) -> Result<(), Box<EvalAltResult>> {
        std::fs::remove_file(path).map_err(|e| e.to_string().into())
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
