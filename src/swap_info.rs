use std::collections::HashMap;
use thiserror::Error;
use procfs::{self, Current};
use procfs::Meminfo;

#[derive(Debug, Clone)]
pub struct ProcessSwapInfo {
    pub pid: i32,
    pub name: String,
    pub swap_size: f64,
}

#[derive(Debug, Clone, Default)]

pub struct SwapUpdate {
    pub aggregated: Vec<ProcessSwapInfo>, 
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(Debug, Clone, Default)]
pub enum SizeUnits {
    #[default]
    KB,
    MB,
    GB
}

#[derive(Debug, Error)]
pub enum SwapDataError {
    #[error("Procfs error: {0}")]
    Procfs(#[from] procfs::ProcError),
    #[error("I/O error accessing /proc: {0}")]
    Io(#[from] std::io::Error),
}


pub fn get_processes_using_swap(unit: SizeUnits) -> Result<Vec<ProcessSwapInfo>, SwapDataError> {
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
                                 let swap_size = convert_swap(swap_kb, unit.clone());
                                 let info = ProcessSwapInfo {
                                     pid,
                                     name,
                                     swap_size,
                                 };
                                 swap_processes.push(info);
                             }
                         }
             			
             		}
            },
            Err(_) => {
                
            }
        }
    }

    Ok(swap_processes)
}

pub fn get_chart_info() -> Result<SwapUpdate, SwapDataError> {
    let process_swap_details = get_processes_using_swap(SizeUnits::KB)?;
    
    let meminfo = Meminfo::current()?;
    
    let total_swap_kb = meminfo.swap_total / 1024;
    let used_swap_kb = meminfo.swap_total.saturating_sub(meminfo.swap_free) / 1024;

    Ok(SwapUpdate {
        aggregated: process_swap_details,
        total_swap: total_swap_kb,
        used_swap: used_swap_kb,
    })
}

pub fn convert_swap(kb: u64, unit: SizeUnits) -> f64 {
    match unit {
        SizeUnits::KB => kb as f64,
        SizeUnits::MB => kb as f64 / 1024.0,
        SizeUnits::GB => kb as f64 / (1024.0 * 1024.0),
    }
}

pub fn aggregate_processes(processes: Vec<ProcessSwapInfo>) -> Vec<ProcessSwapInfo> {
    let mut name_to_info: HashMap<String, (f64, i32)> = HashMap::new();

    for process in processes {
        let entry = name_to_info.entry(process.name).or_insert((0.0, 0));
        entry.0 += process.swap_size;
        entry.1 += 1;
    }

    let mut aggregated_processes: Vec<ProcessSwapInfo> = name_to_info
        .into_iter()
        .map(|(name, (swap_size, count))| ProcessSwapInfo {
            pid: count,  
            name,
            swap_size,
        })
        .collect();

    aggregated_processes.sort_by(|a, b| {
        b.swap_size
            .partial_cmp(&a.swap_size)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    aggregated_processes
}