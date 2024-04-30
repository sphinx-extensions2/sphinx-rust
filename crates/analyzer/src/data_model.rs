//! Data model for the analyzer
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a crate
///
/// .. req:: Represent a crate
///     :id: RUST003
///     :tags: rust
///     :status: in-progress
pub struct Crate {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a module
///
/// .. req:: Represent a module
///     :id: RUST004
///     :tags: rust
///     :status: in-progress
pub struct Module {
    /// The path to the module file
    pub file: Option<String>,
    /// The fully qualified name of the module
    pub path: Vec<String>,
    pub docstring: String,
    /// The public declarations in the module
    pub declarations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a Struct
///
/// .. req:: Represent a struct
///     :id: RUST005
///     :tags: rust
///     :status: in-progress
pub struct Struct {
    /// The fully qualified name of the struct
    pub path: Vec<String>,
    /// The docstring of the struct
    pub docstring: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a Enum
///
/// .. req:: Represent an enum
///     :id: RUST006
///     :tags: rust
///     :status: in-progress
pub struct Enum {
    /// The fully qualified name of the enum
    pub path: Vec<String>,
    /// The docstring of the enum
    pub docstring: String,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a Enum variant
pub struct Variant {
    /// The fully qualified name of the variant
    pub path: Vec<String>,
    /// The docstring of the variant
    pub docstring: String,
    pub discriminant: Option<String>, // TODO shouldn't just be a string
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a Struct or Enum field
pub struct Field {
    /// The fully qualified name of the field.
    ///
    /// Note, for fields of tuple structs, the final component is the index of the field
    pub path: Vec<String>,
    /// The docstring of the field
    pub docstring: String,
    pub type_: TypeSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Representation of a function
pub struct Function {
    /// The fully qualified name of the function.
    pub path: Vec<String>,
    /// The docstring of the function
    pub docstring: String,
    // TODO signature
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A segment of a type signature
///
/// Types are split into segments to allow for easy identification of referenceable elements
pub enum TypeSegment {
    String(String),
    Path(String),
}

/// A representation of a type signature
pub type TypeSignature = Vec<TypeSegment>;
