use memory_rs::external::process::Process;
use windows_sys::Win32::Foundation::GetLastError;
use windows_sys::Win32::System::Diagnostics::Debug::FormatMessageW;
use windows_sys::Win32::System::LibraryLoader::FreeLibrary;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleA;
use windows_sys::Win32::System::LibraryLoader::GetProcAddress;
use windows_sys::Win32::System::Memory::MEM_COMMIT;
use windows_sys::Win32::System::Memory::PAGE_READWRITE;
use windows_sys::Win32::System::Memory::VirtualAllocEx;
use windows_sys::Win32::System::Threading::CreateRemoteThread;
use std::ffi::CString;
use std::ffi::OsString;
use std::ffi::c_void;
use std::os::windows::prelude::*;
use std::ptr;
use std::path::Path;

pub fn inject_dll(process: &Process, name: &str) {
    let filepath = Path::new(name).canonicalize().unwrap();
    let filepath = filepath.to_string_lossy();
    let filepath = (&filepath[4..]).to_owned();
    let dll_dir: Vec<u8> = filepath
        .encode_utf16()
        .flat_map(|x| vec![(x & 0xFF) as u8, ((x & 0xFF00) >> 8) as u8])
        .collect();

    unsafe {
        // Load kernel32 module in order to get LoadLibraryA
        let s_module_handle = CString::new("Kernel32").unwrap();
        let module_handle = GetModuleHandleA(s_module_handle.as_ptr() as _);

        // Load LoadLibraryW function from kernel32 module
        let s_loadlib = CString::new("LoadLibraryW").unwrap();
        let result = GetProcAddress(module_handle, s_loadlib.as_ptr() as _).unwrap();

        // Allocate the space to write the dll direction in the target process
        let addr = VirtualAllocEx(
            process.h_process,
            ptr::null_mut(),
            dll_dir.len(),
            MEM_COMMIT,
            PAGE_READWRITE,
        ) as usize;

        process.write_aob(addr, &dll_dir, true);

        println!("DLL address {:x}", addr);

        let a = CreateRemoteThread(
            process.h_process,
            ptr::null_mut(),
            0,
            Some(std::mem::transmute(result)),
            addr as *const c_void,
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
