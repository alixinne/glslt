//! Python module interface for the GLSLT compiler

use std::path::PathBuf;

use pyo3::exceptions::RuntimeError;
use pyo3::prelude::*;

use glsl::parser::Parse;
use glsl::syntax::TranslationUnit;

use crate::transform::{MinUnit, TransformUnit, Unit};

/// GLSL translation unit
///
/// This represents the syntax tree of an entire GLSL shader stage.
#[pyclass(name = TranslationUnit)]
#[derive(Debug, Clone)]
pub struct PyTranslationUnit {
    tu: TranslationUnit,
}

impl From<TranslationUnit> for PyTranslationUnit {
    fn from(tu: TranslationUnit) -> Self {
        Self { tu }
    }
}

#[pymethods]
impl PyTranslationUnit {
    /// Transform this abstract syntax tree into the corresponding GLSL source code
    #[text_signature = "($self)"]
    pub fn to_glsl(&self) -> PyResult<String> {
        let mut r = String::new();
        glsl::transpiler::glsl::show_translation_unit(
            &mut r,
            &self.tu,
            glsl::transpiler::glsl::FormattingState::default(),
        )
        .map_err(|e| RuntimeError::py_err(format!("{}", e)))?;
        Ok(r)
    }
}

trait HasTransformUnit {
    fn unit(&self) -> &dyn TransformUnit;
    fn unit_mut(&mut self) -> &mut dyn TransformUnit;
}

trait HasTransformUnitExt {
    fn add_unit(&mut self, unit: PyTranslationUnit) -> PyResult<()>;
}

impl<T: HasTransformUnit> HasTransformUnitExt for T {
    fn add_unit(&mut self, unit: PyTranslationUnit) -> PyResult<()> {
        for decl in (unit.tu.0).0.into_iter() {
            self.unit_mut()
                .parse_external_declaration(decl)
                .map_err(|e| RuntimeError::py_err(format!("{}", e)))?;
        }

        Ok(())
    }
}

macro_rules! impl_unit {
    ($pyunit:ident => $unit:ident) => {
        impl From<$unit> for $pyunit {
            fn from(unit: $unit) -> Self {
                Self { unit }
            }
        }

        impl HasTransformUnit for $pyunit {
            fn unit(&self) -> &dyn TransformUnit {
                &self.unit
            }

            fn unit_mut(&mut self) -> &mut dyn TransformUnit {
                &mut self.unit
            }
        }

        #[pymethods]
        impl $pyunit {
            /// Create a new transform unit
            #[new]
            pub fn new() -> Self {
                Self::default()
            }

            /// Add a translation unit's declarations to the current transform unit
            #[text_signature = "($self, unit, /)"]
            pub fn add_unit(&mut self, unit: PyTranslationUnit) -> PyResult<()> {
                <Self as HasTransformUnitExt>::add_unit(self, unit)
            }
        }
    };
}

/// Represents a GLSLT transform unit
#[pyclass(name = Unit)]
#[derive(Default, Debug, Clone)]
pub struct PyUnit {
    unit: Unit,
}

impl_unit!(PyUnit => Unit);

#[pymethods]
impl PyUnit {
    /// Transform this unit into a translation unit (GLSL syntax tree)
    #[text_signature = "($self, /)"]
    pub fn to_translation_unit(&self) -> PyResult<PyTranslationUnit> {
        Ok(self
            .unit
            .clone()
            .into_translation_unit()
            .map_err(|e| RuntimeError::py_err(format!("{}", e)))
            .map(Into::into)?)
    }
}

/// Represents a minifying GLSLT transform unit
#[pyclass(name = MinUnit)]
#[derive(Default, Debug, Clone)]
pub struct PyMinUnit {
    unit: MinUnit,
}

impl_unit!(PyMinUnit => MinUnit);

#[pymethods]
impl PyMinUnit {
    /// Transform this unit into a translation unit (GLSL syntax tree)
    ///
    /// # Parameters
    ///
    /// * `wanted`: list of function names to be included in the dependency tree
    #[text_signature = "($self, wanted, /)"]
    pub fn to_translation_unit(&self, wanted: Vec<String>) -> PyResult<PyTranslationUnit> {
        Ok(self
            .unit
            .clone()
            .into_translation_unit(wanted.iter().map(|s| s.as_str()))
            .map_err(|e| RuntimeError::py_err(format!("{}", e)))
            .map(Into::into)?)
    }
}

#[pymodule]
fn glslt(_py: Python, m: &PyModule) -> PyResult<()> {
    /// Parse a string as GLSL source
    ///
    /// # Parameters
    ///
    /// * `source`: source code to parse
    #[pyfn(m, "parse_string")]
    #[text_signature = "(source, /)"]
    pub fn parse_string_py(_py: Python, source: &str) -> PyResult<PyTranslationUnit> {
        TranslationUnit::parse(source)
            .map(Into::into)
            .map_err(|e| RuntimeError::py_err(format!("{}", e)))
    }

    /// Parse a set of input files into an abstract syntax tree
    ///
    /// # Parameters
    ///
    /// * `files`: list of file names to parse
    /// * `include_paths`: list of system include directories
    #[pyfn(m, "parse_files")]
    #[text_signature = "(files, include_paths, /)"]
    pub fn parse_files_py(
        _py: Python,
        files: Vec<String>,
        include_paths: Vec<String>,
    ) -> PyResult<PyTranslationUnit> {
        crate::parse_files(
            &files.into_iter().map(PathBuf::from).collect::<Vec<_>>(),
            &include_paths
                .into_iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>(),
        )
        .map(Into::into)
        .map_err(|e| RuntimeError::py_err(format!("{}", e)))
    }

    m.add_class::<PyTranslationUnit>()?;
    m.add_class::<PyUnit>()?;
    m.add_class::<PyMinUnit>()?;

    Ok(())
}
