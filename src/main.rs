use memory_rs::external::process::Process;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Not enough arguments.");
        return;
    }

    let p_name = args.get(1).unwrap();
    let dll_name = args.get(2).unwrap();
    let process = Process::new(&p_name).unwrap();

    simple_injector::injector::inject_dll(&process, &dll_name);
}
