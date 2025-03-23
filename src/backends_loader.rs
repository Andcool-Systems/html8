use compiler_core::backend::{Backend, BackendInfo};
use compiler_core::types::NodeType;
use libloading::{Library, Symbol};
use std::collections::HashMap;
use std::path::Path;

pub struct DynBackend {
    _lib: Library,
    pub backend: Box<dyn Backend>,
    pub info: BackendInfo,
    pub cached_code: Option<String>,
}

impl DynBackend {
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        unsafe {
            let lib = Library::new(path.as_ref())
                .map_err(|e| format!("Не удалось загрузить библиотеку: {}", e))?;
            let create_backend: Symbol<unsafe extern "C" fn() -> *mut dyn Backend> = lib
                .get(b"create_backend")
                .map_err(|e| format!("Не удалось найти create_backend: {}", e))?;
            let get_backend_info: Symbol<unsafe extern "C" fn() -> *mut BackendInfo> = lib
                .get(b"get_backend_info")
                .map_err(|e| format!("Не удалось найти get_backend_info: {}", e))?;

            let backend_info_ptr = get_backend_info();
            if backend_info_ptr.is_null() {
                return Err("get_backend_info вернул null".to_string());
            }
            let backend_info = Box::from_raw(backend_info_ptr);
            let backend = Box::from_raw(create_backend());

            Ok(DynBackend {
                _lib: lib,
                backend,
                info: *backend_info,
                cached_code: None,
            })
        }
    }
}

impl Backend for DynBackend {
    fn generate_code(&mut self, node: &NodeType) -> String {
        let code = self.backend.generate_code(node);
        self.cached_code = Some(code.clone());
        code
    }

    fn save_code(&self, path: Option<&str>) -> Result<String, String> {
        let path = path.unwrap_or(&self.info.input_file);
        if let Some(ref code) = self.cached_code {
            std::fs::write(path, code).map_err(|e| format!("Ошибка записи кода: {}", e))?;
            Ok(path.to_string())
        } else {
            Err("Нет сгенерированного кода для сохранения".to_string())
        }
    }

    fn compile(&self) -> Result<(), String> {
        if self.cached_code.is_none() {
            return Err("Код не был сохранён, компиляция невозможна".to_string());
        }

        let args: Vec<String> = self
            .info
            .compiler_args
            .iter()
            .map(|arg| {
                arg.replace("{input}", &self.info.input_file)
                    .replace("{output}", &self.info.output_file)
            })
            .collect();

        let status = std::process::Command::new(&self.info.compiler)
            .args(args)
            .status()
            .map_err(|e| format!("Ошибка запуска компилятора: {}", e))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("Компилятор вернул ошибку: {:?}", status))
        }
    }

    fn run(&self) -> Result<String, String> {
        if self.cached_code.is_none() {
            return Err("Код не был сгенерирован или сохранён, запуск невозможен".to_string());
        }

        let status = std::process::Command::new(&self.info.output_file)
            .output()
            .map_err(|e| format!("Ошибка запуска программы: {}", e))?;

        if status.status.success() {
            Ok(String::from_utf8_lossy(&status.stdout).to_string())
        } else {
            Err(format!("Ошибка выполнения программы: {:?}", status))
        }
    }

    fn supports_feature(&self, feature: &str) -> bool {
        self.backend.supports_feature(feature)
    }
}

pub(crate) fn load_backends(dir: &str) -> HashMap<String, DynBackend> {
    let mut backends = HashMap::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path
                .extension()
                .map_or(false, |ext| ext == "so" || ext == "dll" || ext == "dylib")
            {
                if let Ok(backend) = DynBackend::load(&path) {
                    backends.insert(backend.info.name.clone(), backend);
                }
            }
        }
    }
    backends
}
