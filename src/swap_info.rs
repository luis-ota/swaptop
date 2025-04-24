use thiserror::Error;
use procfs::{self, Current};
use procfs::Meminfo;

#[derive(Debug, Clone)]
pub struct ProcessSwapInfo {
    pub pid: i32,
    pub name: String,
    pub swap_kb: u64,
}

#[derive(Debug, Clone)]

pub struct SwapUpdate {
    pub aggregated: Vec<ProcessSwapInfo>, 
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(Debug, Error)]
pub enum SwapDataError {
    #[error("Procfs error: {0}")]
    Procfs(#[from] procfs::ProcError),
    #[error("I/O error accessing /proc: {0}")]
    Io(#[from] std::io::Error),
}


pub fn get_processes_using_swap() -> Result<Vec<ProcessSwapInfo>, SwapDataError> {
    let mut swap_processes = Vec::new();

    for process_result in procfs::process::all_processes()? {
        match process_result {
            Ok(process) => {
            	let pid = process.pid;
            	if let Ok(status) = process.status() {
                         if let Some(swap_kb) = status.vmswap {
                             if swap_kb > 0 {
                                 let name = match process.stat() {
                                     Ok(stat) => stat.comm,
                                     Err(_) => {
                                         "unknown".to_string()
                                     }
                                 };

                                 let info = ProcessSwapInfo {
                                     pid,
                                     name,
                                     swap_kb,
                                 };
                                 swap_processes.push(info);
                             }
                         }
             			
             		}
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
    
    Ok(swap_processes)
}

pub fn chart_info() -> Result<SwapUpdate, SwapDataError> {
    let process_swap_details = get_processes_using_swap()?;
    
    let meminfo = Meminfo::current()?;
    
    let total_swap_kb = meminfo.swap_total;
    let free_swap_kb = meminfo.swap_free;
    let used_swap_kb = total_swap_kb.saturating_sub(free_swap_kb);

    Ok(SwapUpdate {
        aggregated: process_swap_details,
        total_swap: total_swap_kb,
        used_swap: used_swap_kb,
    })
}