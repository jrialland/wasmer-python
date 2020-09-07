use crate::wasmer;
use pyo3::{exceptions::RuntimeError, prelude::*};
use std::sync::Arc;

/// JIT engine for Wasmer compilers.
///
/// Given an option compiler, it generates the compiled machine code,
/// and publishes it into memory so it can be used externally.
///
/// If the compiler is absent, it will generate a headless engine.
#[pyclass(unsendable)]
#[text_signature = "(/, compiler)"]
pub struct JIT {
    inner: wasmer::JITEngine,
}

impl JIT {
    pub(crate) fn inner(&self) -> &wasmer::JITEngine {
        &self.inner
    }
}

#[pymethods]
impl JIT {
    #[new]
    fn new(compiler: Option<&PyAny>) -> PyResult<Self> {
        Ok(Self {
            inner: match compiler {
                None => wasmer::JIT::headless().engine(),
                Some(compiler) => {
                    let opaque_compiler = compiler.call_method0("into_opaque_compiler")?;
                    let opaque_compiler_inner_ptr = opaque_compiler
                        .call_method0("__inner_as_ptr")?
                        .extract::<usize>()?;

                    let opaque_compiler_inner_ptr: *const OpaqueCompilerInner =
                        opaque_compiler_inner_ptr as _;

                    let opaque_compiler_inner_ref: &OpaqueCompilerInner = unsafe {
                        opaque_compiler_inner_ptr.as_ref().ok_or_else(|| {
                            RuntimeError::py_err(
                                "Failed to transfer the opaque compiler from the compiler",
                            )
                        })?
                    };

                    let opaque_compiler_inner: OpaqueCompilerInner =
                        opaque_compiler_inner_ref.clone();

                    wasmer::JIT::new(opaque_compiler_inner.compiler_config.as_ref()).engine()
                }
            },
        })
    }
}

/// Native engine for Wasmer compilers.
///
/// Given an option compiler, it generates a shared object file
/// (`.so`, `.dylib` or `.dll` depending on the target), saves it
/// temporarily to disk and uses it natively via `dlopen` and `dlsym`.
/// and publishes it into memory so it can be used externally.
///
/// If the compiler is absent, it will generate a headless engine.
#[pyclass(unsendable)]
#[text_signature = "(/, compiler)"]
pub struct Native {
    inner: wasmer::NativeEngine,
}

impl Native {
    pub(crate) fn inner(&self) -> &wasmer::NativeEngine {
        &self.inner
    }
}

#[derive(Clone)]
struct OpaqueCompilerInner {
    compiler_config: Arc<dyn wasmer_compiler::CompilerConfig + Send + Sync>,
}

/// Opaque compiler.
///
/// Internal use only.
#[pyclass]
pub struct OpaqueCompiler {
    inner: OpaqueCompilerInner,
}

impl OpaqueCompiler {
    pub fn raw_with_compiler<C>(compiler_config: C) -> Self
    where
        C: wasmer_compiler::CompilerConfig + Send + Sync + 'static,
    {
        Self {
            inner: OpaqueCompilerInner {
                compiler_config: Arc::new(compiler_config),
            },
        }
    }
}

#[pymethods]
impl OpaqueCompiler {
    pub fn __inner_as_ptr(&self) -> usize {
        let inner_ptr: *const OpaqueCompilerInner = &self.inner;
        let inner_usize: usize = inner_ptr as _;

        inner_usize
    }
}
