use libloading::{Library, Symbol};

pub struct UclLibrary {
    library: Library,
}

impl UclLibrary {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let library = unsafe { Library::new(path)? };
        Ok(Self { library })
    }

    pub fn decompress(&self, input: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        unsafe {
            // Try different UCL decompression functions
            let decompress_functions = [
                "ucl_nrv2d_decompress_8",
                "ucl_nrv2e_decompress_8", 
                "ucl_nrv2f_decompress_8",
                "ucl_decompress",
            ];

            for func_name in &decompress_functions {
                if let Ok(decompress_fn) = self.library.get::<DecompressFn>(func_name.as_bytes()) {
                    return self.try_decompress(input, decompress_fn);
                }
            }

            Err("No compatible UCL decompression function found".into())
        }
    }

    unsafe fn try_decompress(&self, input: &[u8], decompress_fn: Symbol<DecompressFn>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut out_size = 0u32;
        let mut out_len = 0usize;
        
        // First call to get output size
        let result = decompress_fn(
            input.as_ptr(),
            input.len(),
            std::ptr::null_mut(),
            &mut out_len,
            &mut out_size
        );
        
        if result != 0 {
            return Err("UCL decompression failed - first call".into());
        }
        
        // Allocate output buffer
        let mut output = vec![0u8; out_len];
        
        // Second call to actually decompress
        let result = decompress_fn(
            input.as_ptr(),
            input.len(),
            output.as_mut_ptr(),
            &mut out_len,
            &mut out_size
        );
        
        if result != 0 {
            return Err("UCL decompression failed - second call".into());
        }
        
        output.truncate(out_len);
        Ok(output)
    }
}

type DecompressFn = unsafe extern "C" fn(
    *const u8,    // input buffer
    usize,        // input length
    *mut u8,      // output buffer
    *mut usize,   // output length
    *mut u32      // output size
) -> i32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ucl_library_loading() {
        if let Ok(_lib) = UclLibrary::new("unucl/libucl-1.dll") {
            println!("UCL library loaded successfully");
        } else {
            println!("UCL library not found - this is expected if the DLL is not present");
        }
    }
} 