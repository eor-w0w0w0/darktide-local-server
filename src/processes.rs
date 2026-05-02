// use sysinfo::{Pid, Process, ProcessExt, System, SystemExt};
use lazy_static::lazy_static;
use std::sync::Mutex;
use sysinfo::{Pid, ProcessExt, System, SystemExt};
use winapi::um::{
    handleapi::CloseHandle, processthreadsapi::OpenProcess, processthreadsapi::TerminateProcess,
    winnt::PROCESS_TERMINATE,
};

lazy_static! {
    static ref PROCESS_SYSTEM: Mutex<System> = Mutex::new(System::new());
}

fn with_process_system<T>(callback: impl FnOnce(&System) -> T) -> T {
    let mut sys = PROCESS_SYSTEM.lock().unwrap();
    sys.refresh_processes();
    callback(&sys)
}

pub fn is_darktide_running() -> bool {
    with_process_system(|sys| {
        sys.processes()
            .values()
            .any(|process| process.name().eq_ignore_ascii_case("Darktide.exe"))
    })
}

pub fn is_process_running(pid: Pid) -> bool {
    with_process_system(|sys| sys.process(pid).is_some())
}

pub fn stop_process(pid: u32) -> bool {
    unsafe {
        let h_process = OpenProcess(PROCESS_TERMINATE, 0, pid);

        if h_process.is_null() {
            return false;
        }

        let success = TerminateProcess(h_process, 1);
        CloseHandle(h_process);

        success != 0
    }
}
