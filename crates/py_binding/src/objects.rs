use pyo3::prelude::*;

use analyzer::analyze;

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a crate
pub struct Crate {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub version: String,
    #[pyo3(get)]
    pub docstring: String,
}

#[pymethods]
impl Crate {
    pub fn __repr__(&self) -> String {
        format!("Crate(name={:?}, version={:?})", self.name, self.version)
    }
    #[getter]
    pub fn path(&self) -> Vec<String> {
        vec![self.name.clone()]
    }
    #[getter]
    pub fn path_str(&self) -> String {
        self.name.clone()
    }
}

impl From<analyze::Crate> for Crate {
    fn from(crate_: analyze::Crate) -> Self {
        Crate {
            name: crate_.name,
            version: crate_.version,
            docstring: crate_.docstring,
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a module
pub struct Module {
    #[pyo3(get)]
    pub path: Vec<String>,
    #[pyo3(get)]
    pub docstring: String,
}

#[pymethods]
impl Module {
    pub fn __repr__(&self) -> String {
        format!("Module({:?})", self.path_str())
    }
    #[getter]
    pub fn path_str(&self) -> String {
        self.path.join("::")
    }
    #[getter]
    pub fn name(&self) -> String {
        self.path.last().unwrap().clone()
    }
}

impl From<analyze::Module> for Module {
    fn from(module: analyze::Module) -> Self {
        Module {
            path: module.path,
            docstring: module.docstring,
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a struct field
pub struct Field {
    #[pyo3(get)]
    pub path: Vec<String>,
    #[pyo3(get)]
    pub docstring: String,
    #[pyo3(get)]
    pub type_: Vec<TypeSegment>,
}

#[pymethods]
impl Field {
    pub fn __repr__(&self) -> String {
        format!("Field({:?})", self.path_str())
    }
    #[getter]
    pub fn path_str(&self) -> String {
        self.path.join("::")
    }
    #[getter]
    pub fn name(&self) -> String {
        self.path.last().unwrap().clone()
    }
}

impl From<analyze::Field> for Field {
    fn from(field: analyze::Field) -> Self {
        Field {
            path: field.path,
            docstring: field.docstring,
            type_: field.type_.into_iter().map(TypeSegment::from).collect(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a struct
pub struct Struct {
    #[pyo3(get)]
    pub path: Vec<String>,
    #[pyo3(get)]
    pub docstring: String,
    #[pyo3(get)]
    pub fields: Vec<Field>,
}

#[pymethods]
impl Struct {
    pub fn __repr__(&self) -> String {
        format!("Struct({:?})", self.path_str())
    }
    #[getter]
    fn path_str(&self) -> String {
        self.path.join("::")
    }
    #[getter]
    pub fn name(&self) -> String {
        self.path.last().unwrap().clone()
    }
}

impl From<analyze::Struct> for Struct {
    fn from(module: analyze::Struct) -> Self {
        Struct {
            path: module.path,
            docstring: module.docstring,
            fields: module.fields.into_iter().map(Field::from).collect(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of an enum
pub struct Enum {
    #[pyo3(get)]
    pub path: Vec<String>,
    #[pyo3(get)]
    pub docstring: String,
    #[pyo3(get)]
    pub variants: Vec<Variant>,
}

#[pymethods]
impl Enum {
    pub fn __repr__(&self) -> String {
        format!("Enum({:?})", self.path_str())
    }
    #[getter]
    fn path_str(&self) -> String {
        self.path.join("::")
    }
    #[getter]
    pub fn name(&self) -> String {
        self.path.last().unwrap().clone()
    }
}

impl From<analyze::Enum> for Enum {
    fn from(module: analyze::Enum) -> Self {
        Enum {
            path: module.path,
            docstring: module.docstring,
            variants: module.variants.into_iter().map(Variant::from).collect(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of an enum variant
pub struct Variant {
    #[pyo3(get)]
    pub path: Vec<String>,
    #[pyo3(get)]
    pub docstring: String,
    // TODO discriminant
    #[pyo3(get)]
    pub fields: Vec<Field>,
}

#[pymethods]
impl Variant {
    pub fn __repr__(&self) -> String {
        format!("Variant({:?})", self.path_str())
    }
    #[getter]
    fn path_str(&self) -> String {
        self.path.join("::")
    }
    #[getter]
    pub fn name(&self) -> String {
        self.path.last().unwrap().clone()
    }
}

impl From<analyze::Variant> for Variant {
    fn from(var: analyze::Variant) -> Self {
        Variant {
            path: var.path,
            docstring: var.docstring,
            fields: var.fields.into_iter().map(Field::from).collect(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// pyo3 representation of a segment of a type
/// types are split into segments to allow for identification of referenceable elements
pub struct TypeSegment {
    #[pyo3(get)]
    pub content: String,
    #[pyo3(get)]
    pub is_path: bool,
}

#[pymethods]
impl TypeSegment {
    pub fn __repr__(&self) -> String {
        if self.is_path {
            format!("ref({:?})", self.content)
        } else {
            format!("{:?}", self.content)
        }
    }
}

impl From<analyze::TypeSegment> for TypeSegment {
    fn from(field: analyze::TypeSegment) -> Self {
        match field {
            analyze::TypeSegment::Path(content) => TypeSegment {
                content,
                is_path: true,
            },
            analyze::TypeSegment::String(content) => TypeSegment {
                content,
                is_path: false,
            },
        }
    }
}
