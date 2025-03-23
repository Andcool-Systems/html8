use std::fmt;

#[derive(Debug)]
pub struct DefinitionNotFound {
    pub var_name: String,
}

impl DefinitionNotFound {
    pub fn new(name: &str) -> DefinitionNotFound {
        DefinitionNotFound {
            var_name: name.to_string(),
        }
    }
}

impl fmt::Display for DefinitionNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Variable {} not defined", self.var_name)
    }
}

impl std::error::Error for DefinitionNotFound {}
