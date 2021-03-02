use memory_rs::external::process::Process;
use std::ffi::CString;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::mem;
use std::ptr;
use winapi::shared::basetsd::DWORD_PTR;
use winapi::shared::minwindef::LPVOID;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::{FreeLibrary, GetModuleHandleA, GetProcAddress};
use winapi::um::memoryapi::VirtualAllocEx;
use winapi::um::processthreadsapi::CreateRemoteThread;
use winapi::um::winbase::FormatMessageW;
use winapi::um::winnt::{MEM_COMMIT, PAGE_READWRITE};

pub fn inject_dll(process: &Process, name: &str) {
    let dll_dir: Vec<u8> = name
        .to_string()
        .encode_utf16()
        .flat_map(|x| vec![(x & 0xFF) as u8, ((x & 0xFF00) >> 8) as u8])
        .collect();

    unsafe {
        // Load kernel32 module in order to get LoadLibraryA
        let s_module_handle = CString::new("Kernel32").unwrap();
        let module_handle = GetModuleHandleA(s_module_handle.as_ptr());

        // Load LoadLibraryW function from kernel32 module
        let s_loadlib = CString::new("LoadLibraryW").unwrap();
        let result = GetProcAddress(module_handle, s_loadlib.as_ptr());
        assert!(result as usize != 0x0);
        let casted_function: extern "system" fn(LPVOID) -> u32 = mem::transmute(result);

        // Allocate the space to write the dll direction in the target process
        let addr = VirtualAllocEx(
            process.h_process,
            ptr::null_mut(),
            dll_dir.len(),
            MEM_COMMIT,
            PAGE_READWRITE,
        ) as DWORD_PTR;

        process.write_aob(addr, &dll_dir, true);

        println!("DLL address {:x}", addr);

        let a = CreateRemoteThread(
            process.h_process,
            ptr::null_mut(),
            0,
            Some(casted_function),
            addr as LPVOID,
            0,
            ptr::null_mut(),
        );
        println!("handle {:x?}", a);

        let last_err = GetLastError();
        let mut buffer: Vec<u16> = vec![0; 64];
        FormatMessageW(
            0x1000,
            std::ptr::null(),
            last_err,
            0,
            buffer.as_mut_ptr(),
            64,
            std::ptr::null_mut(),
        );
        let msg = OsString::from_wide(&buffer)
            .into_string()
            .unwrap_or("Couldn't parse the error string".to_string());

        println!("Error: {}", msg);

        FreeLibrary(module_handle);
    }
}
