//! Owned, prebuilt CUDA module input. This module does not compile CUDA source
//! or depend on NVRTC. PTX text is CUDA assembly, not CUDA C/C++ source.

#[cfg(feature = "nvrtc")]
use core::ffi::{c_char, CStr};
#[cfg(feature = "nvrtc")]
use std::borrow::ToOwned;
use std::{path::PathBuf, string::String, vec::Vec};

/// An owned prebuilt PTX program, module file, or binary CUBIN.
///
/// The optional historical NVRTC API returns this same carrier type; using
/// this type or its constructors does not enable that compiler feature.
#[derive(Debug, Clone)]
pub struct Ptx(pub(crate) PtxKind);

/// Invalid host-side module input. Validation performs no CUDA operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtxInputError {
    InteriorNul,
    NonUtf8Path,
    EmptyBinary,
}

impl Ptx {
    /// Creates a Ptx from a precompiled PTX or CUBIN file.
    pub fn from_file<P: Into<PathBuf>>(path: P) -> Self {
        Self(PtxKind::File(path.into()))
    }

    /// Creates a Ptx from precompiled PTX assembly text.
    pub fn from_src<S: Into<String>>(src: S) -> Self {
        Self(PtxKind::Src(src.into()))
    }

    /// Creates a Ptx from binary CUBIN data.
    pub fn from_binary(data: Vec<u8>) -> Self {
        Self(PtxKind::Binary(data))
    }

    /// Validates the carrier's host-side metadata before CUDA context binding.
    /// The CUDA driver still validates the PTX/CUBIN program itself.
    pub fn validate(&self) -> Result<(), PtxInputError> {
        match &self.0 {
            #[cfg(feature = "nvrtc")]
            PtxKind::Image(_) => Ok(()),
            PtxKind::Src(src) => {
                if src.as_bytes().contains(&0) {
                    Err(PtxInputError::InteriorNul)
                } else {
                    Ok(())
                }
            }
            PtxKind::File(path) => {
                let path = path.to_str().ok_or(PtxInputError::NonUtf8Path)?;
                if path.as_bytes().contains(&0) {
                    Err(PtxInputError::InteriorNul)
                } else {
                    Ok(())
                }
            }
            PtxKind::Binary(bytes) => {
                if bytes.is_empty() {
                    Err(PtxInputError::EmptyBinary)
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Get the compiled PTX text as a string.
    ///
    /// # Panics
    ///
    /// Preserves the upstream convenience API: panics for binary CUBIN, an
    /// unreadable file, or non-UTF-8 compiler output. Loading a module does
    /// not call this method.
    pub fn to_src(&self) -> String {
        match &self.0 {
            #[cfg(feature = "nvrtc")]
            PtxKind::Image(bytes) => unsafe { CStr::from_ptr(bytes.as_ptr()) }
                .to_str()
                .expect("Unable to convert bytes to str.")
                .to_owned(),
            PtxKind::Src(src) => src.clone(),
            PtxKind::File(path) => {
                std::fs::read_to_string(path).expect("Unable to read ptx from file.")
            }
            PtxKind::Binary(_) => {
                panic!("Cannot convert binary CUBIN data to string. Use from_src() or from_file() for PTX data.")
            }
        }
    }

    /// Gets compiler-produced image bytes, when created by the optional
    /// historical NVRTC API. Prebuilt source/file/binary inputs return None,
    /// preserving the upstream API.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match &self.0 {
            #[cfg(feature = "nvrtc")]
            PtxKind::Image(bytes) => unsafe {
                Some(std::slice::from_raw_parts(
                    bytes.as_ptr().cast(),
                    bytes.len(),
                ))
            },
            _ => None,
        }
    }
}

impl<S: Into<String>> From<S> for Ptx {
    fn from(value: S) -> Self {
        Self::from_src(value)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PtxKind {
    /// NUL-terminated image created by the optional historical compiler.
    #[cfg(feature = "nvrtc")]
    Image(Vec<c_char>),
    Src(String),
    File(PathBuf),
    Binary(Vec<u8>),
}

#[cfg(test)]
mod tests {
    use super::{Ptx, PtxInputError, PtxKind};

    #[test]
    fn prebuilt_source_conversion_preserves_text() {
        let source = ".version 7.0\n.target sm_80\n.address_size 64\n";
        let ptx: Ptx = source.into();
        assert_eq!(ptx.clone().to_src(), source);
        assert!(ptx.as_bytes().is_none());
        assert_eq!(ptx.validate(), Ok(()));
    }

    #[test]
    fn prebuilt_binary_and_file_preserve_owned_input() {
        let binary = Ptx::from_binary(std::vec![0x7f, b'E', b'L', b'F']);
        assert!(matches!(binary.0, PtxKind::Binary(bytes) if bytes == [0x7f, b'E', b'L', b'F']));
        let file = Ptx::from_file("prebuilt.ptx");
        assert!(
            matches!(file.0, PtxKind::File(path) if path == std::path::Path::new("prebuilt.ptx"))
        );
    }

    #[test]
    fn prebuilt_input_validation_needs_no_cuda_context() {
        assert_eq!(
            Ptx::from_src("text\0hidden").validate(),
            Err(PtxInputError::InteriorNul)
        );
        assert_eq!(
            Ptx::from_file("module\0.ptx").validate(),
            Err(PtxInputError::InteriorNul)
        );
        assert_eq!(
            Ptx::from_binary(std::vec![]).validate(),
            Err(PtxInputError::EmptyBinary)
        );
        assert_eq!(Ptx::from_binary(std::vec![0, 1, 0]).validate(), Ok(()));
    }

    #[cfg(all(unix, feature = "std"))]
    #[test]
    fn prebuilt_input_rejects_non_utf8_path_without_cuda() {
        use std::os::unix::ffi::OsStringExt;
        let path = std::ffi::OsString::from_vec(std::vec![0xff]);
        assert_eq!(
            Ptx::from_file(std::path::PathBuf::from(path)).validate(),
            Err(PtxInputError::NonUtf8Path)
        );
    }
}
