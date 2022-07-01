use core::slice;
use windows::Win32::Foundation::CHAR;
use windows::core::PSTR;

/// Allows conversions to Vec<CHAR> in order to facilitate comparisons with arrays of CHAR as defined in the windows crate.
/// This is useful since some struct definitions in the windows crate use CHAR arrays as members to represent strings.
pub trait AsVecOfWinChar {

    fn as_vec_of_win_char(&self) -> Vec<CHAR>;

}

impl AsVecOfWinChar for String {

    fn as_vec_of_win_char(&self) -> Vec<CHAR> {

        self.as_bytes().iter().map(|x|CHAR(*x)).collect()

    }

}

/// This trait is intended to provide the possibility to convert string types defined within the windows crate (such as PSTR)
/// to Rust's String type.
pub trait AsString {

    unsafe fn as_string(&self) -> String;
    
}

impl AsString for PSTR {

    unsafe fn as_string(&self) -> String {

        let mut i: usize = 0;
        while *(self.0.add(i)) != 0 {
            i += 1;
        }
        let my_vec = slice::from_raw_parts(self.0, i).to_owned();
        
        String::from_utf8(my_vec).unwrap()

    }

}