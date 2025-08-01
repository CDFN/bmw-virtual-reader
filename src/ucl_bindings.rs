use std::convert::TryInto;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use libc::{c_int, c_long, c_short, c_uint, c_void};
use libloading::{Library, Symbol};

const UCL_VERSION: u32 = 0x01_0300;

type UclInit2Fn = unsafe extern "C" fn(
    version: u32,
    short: i32,
    int: i32,
    long: i32,
    ucl_uint32: i32,
    ucl_uint: i32,
    minus_one: i32,
    pchar: i32,
    ucl_voidp: i32,
    ucl_compress_t: i32,
) -> c_int;

type UclDecompressFn = unsafe extern "C" fn(
    src: *const u8,
    src_len: c_uint,
    dst: *mut u8,
    dst_len: *mut c_uint,
    wrkmem: *const c_void,
) -> c_int;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct UclLibrary {
    library: Library,
    init_fn: Option<Symbol<'static, UclInit2Fn>>,
    decompress_fn: Option<Symbol<'static, UclDecompressFn>>,
}

#[derive(Debug, Clone)]
pub enum UclErrorKind {
    GenericError,
    InvalidArgument,
    OutOfMemory,
    NotCompressible,
    InputOverrun,
    OutputOverrun,
    LookbehindOverrun,
    EofNotFound,
    InputNotConsumed,
    OverlapOverrun,
    SrcTooLarge,
    DstTooLarge,
    DstTooSmall,
}

impl std::fmt::Display for UclErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UclErrorKind::GenericError => write!(f, "generic UCL error"),
            UclErrorKind::InvalidArgument => write!(f, "invalid argument"),
            UclErrorKind::OutOfMemory => write!(f, "out of memory"),
            UclErrorKind::NotCompressible => write!(f, "not compressible"),
            UclErrorKind::InputOverrun => write!(f, "input overrun"),
            UclErrorKind::OutputOverrun => write!(f, "output overrun"),
            UclErrorKind::LookbehindOverrun => write!(f, "look-behind overrun"),
            UclErrorKind::EofNotFound => write!(f, "EOF not found"),
            UclErrorKind::InputNotConsumed => write!(f, "input not consumed"),
            UclErrorKind::OverlapOverrun => write!(f, "overlap overrun"),
            UclErrorKind::SrcTooLarge => write!(f, "src buffer too large"),
            UclErrorKind::DstTooLarge => write!(f, "dst buffer too large"),
            UclErrorKind::DstTooSmall => write!(f, "dst buffer too small"),
        }
    }
}

impl std::error::Error for UclErrorKind {}

impl UclErrorKind {
    fn from_code(code: i32) -> Self {
        match code {
            -2 => UclErrorKind::InvalidArgument,
            -3 => UclErrorKind::OutOfMemory,
            -101 => UclErrorKind::NotCompressible,
            -201 => UclErrorKind::InputOverrun,
            -202 => UclErrorKind::OutputOverrun,
            -203 => UclErrorKind::LookbehindOverrun,
            -204 => UclErrorKind::EofNotFound,
            -205 => UclErrorKind::InputNotConsumed,
            -206 => UclErrorKind::OverlapOverrun,
            _ => UclErrorKind::GenericError,
        }
    }
}

impl UclLibrary {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let library = unsafe { Library::new(path)? };
        
        // Load the required functions
        let init_fn = unsafe {
            match library.get::<UclInit2Fn>(b"__ucl_init2") {
                Ok(f) => Some(std::mem::transmute(f)),
                Err(_) => None,
            }
        };
        
        // Try different decompression function names
        let decompress_fn = unsafe {
            let function_names: &[&[u8]] = &[
                b"ucl_nrv2b_decompress_safe_8",
                b"ucl_nrv2d_decompress_safe_8", 
                b"ucl_nrv2e_decompress_safe_8",
                b"ucl_nrv2b_decompress_8",
                b"ucl_nrv2d_decompress_8",
                b"ucl_nrv2e_decompress_8",
            ];
            
            let mut found_fn = None;
            for &func_name in function_names {
                match library.get::<UclDecompressFn>(func_name) {
                    Ok(f) => {
                        found_fn = Some(std::mem::transmute(f));
                        break;
                    }
                    Err(_) => continue,
                }
            }
            found_fn
        };
        
        if decompress_fn.is_none() {
            return Err("No compatible UCL decompression function found in library".into());
        }
        
        let lib = Self {
            library,
            init_fn,
            decompress_fn,
        };
        
        // Initialize UCL library if possible
        lib.ucl_init()?;
        
        Ok(lib)
    }
    
    fn ucl_init(&self) -> Result<(), Box<dyn std::error::Error>> {
        if INITIALIZED.load(Ordering::Acquire) {
            return Ok(());
        }

        if let Some(ref init_fn) = self.init_fn {
            unsafe {
                let res = init_fn(
                    UCL_VERSION,
                    mem::size_of::<c_short>() as i32,
                    mem::size_of::<c_int>() as i32,
                    mem::size_of::<c_long>() as i32,
                    mem::size_of::<u32>() as i32,
                    mem::size_of::<c_uint>() as i32,
                    -1i32,
                    mem::size_of::<*mut u8>() as i32,
                    mem::size_of::<*mut c_void>() as i32,
                    mem::size_of::<*mut c_void>() as i32, // function ptr
                );
                
                if res != 0 {
                    return Err(format!("UCL init failed with code {}. Incompatible library version or architecture?", res).into());
                }
                
                INITIALIZED.store(true, Ordering::Release);
            }
        }
        
        Ok(())
    }

    pub fn decompress(&self, input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Add input validation
        if input.is_empty() {
            return Err("Input data is empty".into());
        }
        
        if input.len() < 4 {
            return Err("Input data too small (less than 4 bytes)".into());
        }
        
        // Check for reasonable input size limits
        if input.len() > 100 * 1024 * 1024 {
            return Err(format!("Input data too large: {} bytes", input.len()).into());
        }

        
        // Try with different buffer sizes, starting with a reasonable estimate
        let buffer_sizes = [
            input.len() * 20,        // 20x compression ratio
            input.len() * 50,        // 50x compression ratio
            input.len() * 100,       // 100x compression ratio
            10 * 1024 * 1024,       // 10MB
            50 * 1024 * 1024,       // 50MB
        ];
        
        for &buffer_size in &buffer_sizes {
            if buffer_size > 200 * 1024 * 1024 {
                continue; // Skip sizes over 200MB
            }
            
            match self.try_decompress_with_size(input, buffer_size) {
                Ok(result) => return Ok(result),
                Err(UclErrorKind::OutputOverrun) => continue,
                Err(e) => return Err(format!("UCL decompression failed: {}", e).into()),
            }
        }
        
        Err("UCL decompression failed: all buffer sizes exhausted".into())
    }
    
    fn try_decompress_with_size(&self, input: &[u8], buffer_size: usize) -> Result<Vec<u8>, UclErrorKind> {
        let decompress_fn = match self.decompress_fn.as_ref() {
            Some(f) => f,
            None => return Err(UclErrorKind::GenericError),
        };
        
        let src_len = match input.len().try_into() {
            Ok(v) => v,
            Err(_) => return Err(UclErrorKind::SrcTooLarge),
        };

        let mut dst = Vec::with_capacity(buffer_size);
        let mut dst_len = buffer_size as c_uint;

        unsafe {
            let res = decompress_fn(
                input.as_ptr(),
                src_len,
                dst.as_mut_ptr(),
                &mut dst_len,
                ptr::null(),
            );

            match res {
                0 => {
                    assert!(
                        dst_len <= (buffer_size as u32),
                        "decompression yielded more data than available in dst buffer"
                    );
                    dst.set_len(dst_len as usize);
                    Ok(dst)
                }
                _ => Err(UclErrorKind::from_code(res)),
            }
        }
    }
}