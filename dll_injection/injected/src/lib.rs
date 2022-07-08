use std::{ptr, ffi::c_void};
use std::str;
use std::path::Path;
use std::error::Error;
use windows::{

    Win32::Foundation::{HINSTANCE, BOOL, MAX_PATH, HANDLE, CloseHandle}, Win32::System::SystemServices::*, Win32::System::LibraryLoader::GetModuleFileNameA, 
    Win32::System::Threading::*, Win32::Storage::FileSystem::{FILE_ACCESS_FLAGS, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_MODE, CreateFileA},

};

unsafe fn try_evil_thread_routine() -> Result<(), Box<dyn Error>> {

    let mut proc_path = [0; MAX_PATH as usize];

    GetModuleFileNameA(HINSTANCE(0), &mut proc_path);
    let proc_path_str = str::from_utf8(&proc_path)?;
    let proc_stem = Path::new(proc_path_str).file_stem()
        .and_then(|f_stem|f_stem.to_str())
        .ok_or("[ERR] Failed to parse the filename.")?;
    let new_file_path = format!("C:\\Users\\Public\\injected_{}!.txt", proc_stem);
    let file_hdl = CreateFileA(new_file_path, FILE_ACCESS_FLAGS(GENERIC_READ | GENERIC_WRITE), 
        FILE_SHARE_MODE(0), ptr::null(), CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 
        HANDLE::default())?;
    CloseHandle(file_hdl);

    Ok(())

}

unsafe extern "system" fn evil_thread_routine(_thread_param: *mut c_void) -> u32 {

    if let Err(_) = try_evil_thread_routine() {
        return 1;
    }

    0

}

#[no_mangle]
pub extern "system" fn DllMain(_dll_handle: HINSTANCE, call_reason: u32, _reserved: *const u8) -> BOOL {

    unsafe {
        if call_reason == DLL_PROCESS_ATTACH {
            let _th_hdl = CreateThread(ptr::null(), 0, 
                Option::Some(evil_thread_routine), ptr::null(), THREAD_CREATE_RUN_IMMEDIATELY, ptr::null_mut());
        }
    }

    BOOL(1)

}