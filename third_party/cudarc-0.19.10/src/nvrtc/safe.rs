//! Safe abstractions around [crate::nvrtc::result] for compiling PTX files.
//!
//! Call [compile_ptx()] or [compile_ptx_with_opts()].

use super::{result, sys};

use core::ffi::CStr;
use std::ffi::CString;
use std::{borrow::ToOwned, string::String, vec::Vec};

// Compatibility export for historical compiler users. The carrier and driver
// loader themselves are available without enabling this compiler module.
pub use crate::ptx::Ptx;
use crate::ptx::PtxKind;

/// Calls [compile_ptx_with_opts] with no options. `src` is the source string
/// of a `.cu` file.
///
/// Example:
/// ```rust
/// # use cudarc::nvrtc::*;
/// let ptx = compile_ptx("extern \"C\" __global__ void kernel() { }").unwrap();
/// ```
pub fn compile_ptx<S: AsRef<str>>(src: S) -> Result<Ptx, CompileError> {
    compile_ptx_with_opts(src, Default::default())
}

/// Compiles `src` with the given `opts`. `src` is the source string of a `.cu` file.
///
/// Example:
/// ```rust
/// # use cudarc::nvrtc::*;
/// let opts = CompileOptions {
///     ftz: Some(true),
///     maxrregcount: Some(10),
///     ..Default::default()
/// };
/// let ptx = compile_ptx_with_opts("extern \"C\" __global__ void kernel() { }", opts).unwrap();
/// ```
pub fn compile_ptx_with_opts<S: AsRef<str>>(
    src: S,
    opts: CompileOptions,
) -> Result<Ptx, CompileError> {
    let prog = Program::create(src, opts.name.as_deref())?;
    prog.compile(opts)
}

pub(crate) struct Program {
    prog: sys::nvrtcProgram,

    // Held in this struct to ensure they are not
    // dropped until after the nvrtcProgram.
    _src: CString,
    _name: Option<CString>,
}

impl Program {
    pub(crate) fn create<S: AsRef<str>>(src: S, name: Option<&str>) -> Result<Self, CompileError> {
        let src = CString::new(src.as_ref().as_bytes())
            .expect("program code cannot contain null terminators");
        let name =
            name.map(|s| CString::new(s).expect("program name cannot contain null terminators"));
        let prog =
            result::create_program(&src, name.as_deref()).map_err(CompileError::CreationError)?;
        Ok(Self {
            prog,
            _src: src,
            _name: name,
        })
    }

    pub(crate) fn compile(self, opts: CompileOptions) -> Result<Ptx, CompileError> {
        let options = opts.build();

        unsafe { result::compile_program(self.prog, &options) }.map_err(|e| {
            let log_raw = unsafe { result::get_program_log(self.prog) }.unwrap();
            let log_ptr = log_raw.as_ptr();
            let log = unsafe { CStr::from_ptr(log_ptr) }.to_owned();
            CompileError::CompileError {
                nvrtc: e,
                options,
                log,
            }
        })?;

        let image = unsafe { result::get_ptx(self.prog) }.map_err(CompileError::GetPtxError)?;

        Ok(Ptx(PtxKind::Image(image)))
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        let prog = std::mem::replace(&mut self.prog, std::ptr::null_mut());
        if !prog.is_null() {
            unsafe { result::destroy_program(prog) }.unwrap()
        }
    }
}

/// Represents an error that happens during nvrtc compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// Error happened during [result::create_program()]
    CreationError(result::NvrtcError),

    /// Error happened during [result::compile_program()]
    CompileError {
        nvrtc: result::NvrtcError,
        options: Vec<String>,
        log: CString,
    },

    /// Error happened during [result::get_program_log()]
    GetLogError(result::NvrtcError),

    /// Error happened during [result::get_ptx()]
    GetPtxError(result::NvrtcError),

    /// Error happened during [result::destroy_program()]
    DestroyError(result::NvrtcError),
}

#[cfg(feature = "std")]
impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CompileError {}

/// Flags you can pass to the nvrtc compiler.
/// See <https://docs.nvidia.com/cuda/nvrtc/index.html#group__options>
/// for all available flags and documentation for what they do.
///
/// All fields of this struct match one of the flags in the documentation.
/// if a field is `None` it will not be passed to the compiler.
///
/// All fields default to `None`.
///
/// *NOTE*: not all flags are currently supported.
///
/// Example:
/// ```rust
/// # use cudarc::nvrtc::*;
/// // "--ftz=true" will be passed to the compiler
/// let opts = CompileOptions {
///     ftz: Some(true),
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct CompileOptions {
    pub ftz: Option<bool>,
    pub prec_sqrt: Option<bool>,
    pub prec_div: Option<bool>,
    pub fmad: Option<bool>,
    pub options: Vec<String>,
    pub use_fast_math: Option<bool>,
    pub maxrregcount: Option<usize>,
    pub include_paths: Vec<String>,
    pub arch: Option<&'static str>,
    pub name: Option<String>,
}

impl CompileOptions {
    pub(crate) fn build(self) -> Vec<String> {
        let mut options: Vec<String> = Vec::new();

        if let Some(v) = self.ftz {
            options.push(std::format!("--ftz={v}"));
        }

        if let Some(v) = self.prec_sqrt {
            options.push(std::format!("--prec-sqrt={v}"));
        }

        if let Some(v) = self.prec_div {
            options.push(std::format!("--prec-div={v}"));
        }

        if let Some(v) = self.fmad {
            options.push(std::format!("--fmad={v}"));
        }

        if let Some(true) = self.use_fast_math {
            options.push("--fmad=true".into());
        }

        if let Some(count) = self.maxrregcount {
            options.push(std::format!("--maxrregcount={count}"));
        }

        for path in self.include_paths {
            options.push(std::format!("--include-path={path}"));
        }

        if let Some(arch) = self.arch {
            options.push(std::format!("--gpu-architecture={arch}"))
        }

        for option in self.options {
            options.push(option);
        }

        options
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_no_opts() {
        const SRC: &str =
            "extern \"C\" __global__ void sin_kernel(float *out, const float *inp, int numel) {
            int i = blockIdx.x * blockDim.x + threadIdx.x;
            if (i < numel) {
                out[i] = sin(inp[i]);
            }
        }";
        compile_ptx_with_opts(SRC, Default::default()).unwrap();
    }

    #[test]
    fn test_compile_options_build_none() {
        let opts: CompileOptions = Default::default();
        assert!(opts.build().is_empty());
    }

    #[test]
    fn test_compile_options_build_ftz() {
        let opts = CompileOptions {
            ftz: Some(true),
            ..Default::default()
        };
        assert_eq!(&opts.build(), &["--ftz=true"]);
    }

    #[test]
    fn test_compile_options_build_multi() {
        let opts = CompileOptions {
            prec_div: Some(false),
            maxrregcount: Some(60),
            ..Default::default()
        };
        assert_eq!(&opts.build(), &["--prec-div=false", "--maxrregcount=60"]);
    }
}
