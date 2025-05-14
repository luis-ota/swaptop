use std::collections::HashMap;
use thiserror::Error;

#[cfg(target_os = "windows")]
use std::io;

#[cfg(feature = "linux")]
use procfs::{self, Current};
#[cfg(feature = "linux")]
use procfs::Meminfo;


#[derive(Debug, Clone)]
pub struct ProcessSwapInfo {
    pub pid: u32,
    pub name: String,
    pub swap_size: f64,
}

#[derive(Debug, Clone, Default)]

pub struct SwapUpdate {
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

#[cfg(target_os = "linux")]
#[derive(Debug, Error)]
pub enum SwapDataError {
    #[error("Procfs error: {0}")]
    Procfs(#[from] procfs::ProcError),
    #[error("I/O error accessing /proc: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(feature = "windows_support")]
#[cfg(target_os = "windows")]
#[derive(Debug, Error)]
pub enum SwapDataError {
    #[error("I/O error accessing system information: {0}")]
    Io(#[from] io::Error),
}
#[cfg(feature = "linux")]
#[cfg(target_os = "linux")]
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
                                     pid: pid as u32,
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

#[cfg(target_os = "windows")]
pub fn get_processes_using_swap(unit: SizeUnits) -> Result<Vec<ProcessSwapInfo>, SwapDataError> {
    let mut profile_page_processes = Vec::new();

        match tasklist::Tasklist::new() {
            Ok(tasks) => {
                for task in tasks{
                    let meminfo = task.get_memory_info();

                    let info = ProcessSwapInfo{
                        pid: task.pid,
                        name: task.pname,
                        swap_size: convert_swap(meminfo.get_pagefile_usage() as u64 /1024, unit.clone())
                    };
                    profile_page_processes.push(info);
                }

            }
            Err(_) => {
            }
        
    }

    Ok(profile_page_processes)
}

#[cfg(target_os = "linux")]
pub fn get_chart_info() -> Result<SwapUpdate, SwapDataError> {
    let meminfo = Meminfo::current()?;
    
    let total_swap_kb = meminfo.swap_total / 1024;
    let used_swap_kb = meminfo.swap_total.saturating_sub(meminfo.swap_free) / 1024;

    Ok(SwapUpdate {
        total_swap: total_swap_kb,
        used_swap: used_swap_kb,
    })
}

#[cfg(target_os = "windows")]
pub fn get_chart_info() -> Result<SwapUpdate, SwapDataError> {
    use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use std::mem::MaybeUninit;

    unsafe {
        let mut mem_status = MaybeUninit::<MEMORYSTATUSEX>::zeroed();
        mem_status.as_mut_ptr().write(MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            ..Default::default()
        });

        if GlobalMemoryStatusEx(mem_status.as_mut_ptr()) == 0 {
            return Err(SwapDataError::Io(std::io::Error::last_os_error()));
        }

        let mem_status = mem_status.assume_init();

        // Page file values are in bytes, convert to KB
        let total_swap = mem_status.ullTotalPageFile / 1024;
        let used_swap = (mem_status.ullTotalPageFile - mem_status.ullAvailPageFile) / 1024;

        Ok(SwapUpdate {
            total_swap: total_swap as u64,
            used_swap: used_swap as u64,
        })
    }
}

pub fn convert_swap(kb: u64, unit: SizeUnits) -> f64 {
    match unit {
        SizeUnits::KB => kb as f64,
        SizeUnits::MB => kb as f64 / 1024.0,
        SizeUnits::GB => kb as f64 / (1024.0 * 1024.0),
    }
}

pub fn aggregate_processes(processes: Vec<ProcessSwapInfo>) -> Vec<ProcessSwapInfo> {
    let mut name_to_info: HashMap<String, (f64, u32)> = HashMap::new();

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