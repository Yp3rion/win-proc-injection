use crate::type_conversions::AsVecOfWinChar;
use crate::errors::InjectionError;
use crate::errors::Result;

use std::ffi::c_void;
use std::str;
use std::ptr;
use std::mem;
use windows::{

    Win32::System::Diagnostics::ToolHelp::*, Win32::Foundation::{BOOL, MAX_PATH, CloseHandle, LUID, HANDLE, ERROR_NOT_ALL_ASSIGNED, GetLastError}, 
    Win32::System::Threading::*, Win32::System::Memory::*, Win32::Storage::FileSystem::{GetFullPathNameA, GetFileAttributesA, INVALID_FILE_ATTRIBUTES, FILE_ATTRIBUTE_DIRECTORY},
    core::{PSTR, PCSTR}, Win32::System::WindowsProgramming::INFINITE, Win32::Security::{TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, TOKEN_PRIVILEGES, SE_PRIVILEGE_ENABLED, 
    LookupPrivilegeValueA, AdjustTokenPrivileges}, Win32::System::Diagnostics::Debug::WriteProcessMemory, Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress}, 
    Win32::System::SystemServices::SE_DEBUG_NAME

};

#[derive(Default)]
pub struct Injector {

    target_process : String,
    lib_path : String,

}

impl Injector {

    pub fn new(target_process: String, lib_path: String) -> Self {
        Injector {
            target_process,
            lib_path
        }
    }

    pub fn set_target_process(&mut self, target_process: String) {
        self.target_process = target_process;
    }

    pub fn set_lib_path(&mut self, lib_path: String) {
        self.lib_path = lib_path;
    }
    
    // Tries to enable a given privilege in the current process' access token.
    unsafe fn get_privilege(&self, priv_name: &str) -> Result<()> {

        let mut token_hdl = HANDLE::default();
        let mut priv_luid = LUID::default();
        let mut token_privs = TOKEN_PRIVILEGES::default();

        OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token_hdl)
            .as_bool()
            .then(||()).
            ok_or(InjectionError::NoDebugPriv(None))?;
        LookupPrivilegeValueA(PCSTR::default(), priv_name, &mut priv_luid)
            .as_bool()
            .then(||())
            .ok_or(InjectionError::NoDebugPriv(None))?;

        token_privs.PrivilegeCount = 1;
        token_privs.Privileges[0].Luid = priv_luid;
        token_privs.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        AdjustTokenPrivileges(token_hdl, 
            false, &token_privs, mem::size_of::<TOKEN_PRIVILEGES>() as u32, ptr::null_mut(), ptr::null_mut())
            .as_bool()
            .then(||())
            .ok_or(InjectionError::NoDebugPriv(None))?;

        (GetLastError() != ERROR_NOT_ALL_ASSIGNED).then(||()).ok_or(InjectionError::NoDebugPriv(None))?;

        Ok(())

    }

    // Looks for a process with the same name as the target and returns its PID or 0 if no process is found.
    unsafe fn find_process(&self) -> Result<u32> {

        let target_process_as_char_slice_ref = &self.target_process.as_vec_of_win_char()[..];
        let process_ptr : *mut PROCESSENTRY32 = &mut PROCESSENTRY32::default() as *mut PROCESSENTRY32;
        // Need to initialize the dwSize field with the size of the struct.
        (*process_ptr).dwSize = mem::size_of::<PROCESSENTRY32>() as u32;
        let snapshot_handle = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|err|InjectionError::UnexpectedError(Some(err.into())))?;
        let mut proc_found: BOOL = Process32First(snapshot_handle, process_ptr);

        while proc_found.as_bool() {
            // It is necessary to take a slice because the szExeFile array of CHAR may contain values different from CHAR(0) after
            // the end of the string.
            if (*process_ptr).szExeFile[..self.target_process.len()] == *target_process_as_char_slice_ref {
                CloseHandle(snapshot_handle);
                return Ok((*process_ptr).th32ProcessID);
            }
            proc_found = Process32Next(snapshot_handle, process_ptr);
        }

        Err(InjectionError::ProcNotFound(None))

    }

    unsafe fn prepare_target_process(&self, pid: u32, proc_hdl: &mut HANDLE, mem_hdl: &mut *mut c_void) -> Result<()> {

        *proc_hdl = OpenProcess(PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE, 
            false, pid).map_err(|err|InjectionError::ProcError(Some(err.into())))?;
        *mem_hdl = VirtualAllocEx(*proc_hdl, 
            ptr::null(), MAX_PATH as usize, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
        (!(*mem_hdl).is_null()).then(||()).ok_or(InjectionError::ProcError(None))?;
        let mut full_lib_path = [0; MAX_PATH as usize]; 
        let mut lib_filename = PSTR(ptr::null_mut());
        GetFullPathNameA(self.lib_path.clone(), &mut full_lib_path, &mut lib_filename as *mut PSTR);
        let file_attr = GetFileAttributesA(PCSTR(&full_lib_path as *const u8));
        (file_attr != INVALID_FILE_ATTRIBUTES && (file_attr & FILE_ATTRIBUTE_DIRECTORY.0) == 0)
            .then(||())
            .ok_or(InjectionError::LibNotFound(None))?;
        WriteProcessMemory(*proc_hdl, 
            *mem_hdl, &full_lib_path as *const _ as *const c_void, MAX_PATH as usize, ptr::null_mut())
            .as_bool()
            .then(||())
            .ok_or(InjectionError::ProcError(None))?;

        Ok(())

    }

    unsafe fn trigger_evil_code(&self, proc_hdl: HANDLE, mem_hdl: *const c_void) -> Result<()> {

        let kernel32_hdl = GetModuleHandleA("kernel32.dll").map_err(|err|InjectionError::UnexpectedError(Some(err.into())))?;
        let loadlibrary_ptr = GetProcAddress(kernel32_hdl, "LoadLibraryA")
            .ok_or(InjectionError::UnexpectedError(None))?;
        let loadlibrary_ptr_changed = *(&loadlibrary_ptr as *const _ as *const unsafe extern "system" fn(*mut c_void) -> u32);
        let remote_thread_hdl = CreateRemoteThread(proc_hdl, ptr::null(), 
            0, Some(loadlibrary_ptr_changed), mem_hdl, 0, ptr::null_mut())
            .map_err(|err|InjectionError::ProcError(Some(err.into())))?;
        WaitForSingleObject(remote_thread_hdl, INFINITE);

        Ok(())

    }

    unsafe fn release_target_process(&self, proc_hdl: HANDLE, mem_ptr: *mut c_void) {

        VirtualFreeEx(proc_hdl, mem_ptr, 0, MEM_RELEASE);
        CloseHandle(proc_hdl);

    }

    pub unsafe fn start_target_process(&self) -> Result<()> {
        let startup_info = &mut STARTUPINFOA::default() as *mut STARTUPINFOA;
        let proc_info = &mut PROCESS_INFORMATION::default() as *mut PROCESS_INFORMATION;
        (*startup_info).cb = mem::size_of::<STARTUPINFOA>() as u32;
        CreateProcessA(PCSTR(ptr::null()), PSTR(self.target_process.clone().as_mut_ptr()), 
            ptr::null(), ptr::null(), false, CREATE_NEW_CONSOLE, ptr::null(), 
            PCSTR(ptr::null()), startup_info, proc_info)
            .as_bool()
            .then(||())
            .ok_or(InjectionError::ExeNotFound(None))?;

        Ok(())
    }

    pub unsafe fn inject(&self) -> Result<()> {

        self.get_privilege(SE_DEBUG_NAME).unwrap_or_else(|err|eprintln!("[INFO] {}", err));
        let pid: u32 = self.find_process()?;
        let mut proc_hdl: HANDLE = HANDLE(0);
        let mut mem_ptr: *mut c_void = ptr::null_mut();
        self.prepare_target_process(pid, &mut proc_hdl, &mut mem_ptr)?;
        self.trigger_evil_code(proc_hdl, mem_ptr as *const c_void)?;
        self.release_target_process(proc_hdl, mem_ptr);

        Ok(())

    }

}