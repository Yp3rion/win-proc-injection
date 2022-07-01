use crate::type_conversions::AsVecOfWinChar;

use std::ffi::c_void;
use std::str;
use std::ptr;
use std::panic;
use std::mem;
use windows::{

    Win32::System::Diagnostics::ToolHelp::*, Win32::Foundation::{BOOL, MAX_PATH, ERROR_NOT_ALL_ASSIGNED, CloseHandle, GetLastError, LUID, HANDLE}, 
    Win32::System::Threading::*, Win32::System::Memory::*, Win32::Storage::FileSystem::GetFullPathNameA, core::{PSTR, PCSTR}, Win32::System::WindowsProgramming::INFINITE,
    Win32::Security::{TOKEN_ADJUST_PRIVILEGES, TOKEN_QUERY, TOKEN_PRIVILEGES, SE_PRIVILEGE_ENABLED, LookupPrivilegeValueA, AdjustTokenPrivileges}, 
    Win32::System::Diagnostics::Debug::WriteProcessMemory, Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress}, Win32::System::SystemServices::SE_DEBUG_NAME

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
    
    unsafe fn get_privilege(&self, priv_name: &str) -> bool {
        let mut token_hdl = HANDLE::default();
        let mut priv_luid = LUID::default();
        let mut token_privs = TOKEN_PRIVILEGES::default();

        OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token_hdl);
        LookupPrivilegeValueA(PCSTR::default(), priv_name, &mut priv_luid);

        token_privs.PrivilegeCount = 1;
        token_privs.Privileges[0].Luid = priv_luid;
        token_privs.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        AdjustTokenPrivileges(token_hdl, 
            false, &token_privs, mem::size_of::<TOKEN_PRIVILEGES>() as u32, ptr::null_mut(), ptr::null_mut());

        if GetLastError() == ERROR_NOT_ALL_ASSIGNED {
            return false;
        }

        true
    }

    /// Looks for a process with the same name as the target and returns its PID or 0 if no process is found.
    unsafe fn find_process(&self) -> u32 {

        let target_process_as_char_slice_ref = &self.target_process.as_vec_of_win_char()[..];
        let process_ptr : *mut PROCESSENTRY32 = &mut PROCESSENTRY32::default() as *mut PROCESSENTRY32;
        // Need to initialize the dwSize field with the size of the struct.
        (*process_ptr).dwSize = mem::size_of::<PROCESSENTRY32>() as u32;
        let snapshot_handle_res = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        let snapshot_handle = match snapshot_handle_res {
            Ok(sh) => sh,
            Err(e) => panic!()
        };
        let mut proc_found: BOOL = Process32First(snapshot_handle, process_ptr);

        while proc_found.as_bool() {
            // It is necessary to take a slice because the szExeFile array of CHAR may contain values different from CHAR(0) after
            // the end of the string.
            if (*process_ptr).szExeFile[..self.target_process.len()] == *target_process_as_char_slice_ref {
                return (*process_ptr).th32ProcessID;
            }
            proc_found = Process32Next(snapshot_handle, process_ptr);
        }

        0

    }

    unsafe fn prepare_target_process(&self, pid: u32, proc_hdl: &mut HANDLE, mem_hdl: &mut *mut c_void) {

        *proc_hdl = OpenProcess(PROCESS_CREATE_THREAD | PROCESS_VM_OPERATION | PROCESS_VM_READ | PROCESS_VM_WRITE, 
            false, pid).unwrap();
        *mem_hdl = VirtualAllocEx(*proc_hdl, 
            ptr::null(), MAX_PATH as usize, MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE);
        let mut full_lib_path = [0; MAX_PATH as usize]; 
        let mut lib_filename = PSTR(ptr::null_mut());
        let n_bytes_written = GetFullPathNameA(self.lib_path.clone(), &mut full_lib_path, &mut lib_filename as *mut PSTR);
        //full_lib_path = full_lib_path.as_slice().iter().map(|x|)
        let success = WriteProcessMemory(*proc_hdl, 
            *mem_hdl, &full_lib_path as *const _ as *const c_void, MAX_PATH as usize, ptr::null_mut());

    }

    unsafe fn trigger_evil_code(&self, proc_hdl: HANDLE, mem_hdl: *const c_void) {

        let kernel32_hdl = GetModuleHandleA("kernel32.dll").unwrap();
        let loadlibrary_ptr = GetProcAddress(kernel32_hdl, "LoadLibraryA").unwrap();
        let loadlibrary_ptr_changed = *(&loadlibrary_ptr as *const _ as *const unsafe extern "system" fn(*mut c_void) -> u32);
        let remote_thread_hdl = CreateRemoteThread(proc_hdl, ptr::null(), 
            0, Some(loadlibrary_ptr_changed), mem_hdl, 0, ptr::null_mut());
        WaitForSingleObject(remote_thread_hdl.unwrap(), INFINITE);

    }

    unsafe fn release_target_process(&self, proc_hdl: HANDLE, mem_ptr: *mut c_void) {

        VirtualFreeEx(proc_hdl, mem_ptr, 0, MEM_RELEASE);
        CloseHandle(proc_hdl);

    }

    pub unsafe fn inject(&self) {

        self.get_privilege(SE_DEBUG_NAME);
        let pid: u32 = self.find_process();
        let mut proc_hdl: HANDLE = HANDLE(0);
        let mut mem_ptr: *mut c_void = ptr::null_mut();
        self.prepare_target_process(pid, &mut proc_hdl, &mut mem_ptr);
        self.trigger_evil_code(proc_hdl, mem_ptr as *const c_void);
        self.release_target_process(proc_hdl, mem_ptr);

    }

}