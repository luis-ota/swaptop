use std::collections::HashMap;
use thiserror::Error;

#[cfg(target_os = "windows")]
use std::io;

use proc_mounts::SwapIter;
#[cfg(target_os = "linux")]
use procfs::{self, Current, Meminfo};

#[derive(Debug, Clone)]
pub struct ProcessSwapInfo {
    pub pid: u32,
    pub name: String,
    pub swap_size: f64,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub struct InfoSwap {
    pub name: String,
    pub kind: String,
    pub size_kb: u64,
    pub used_kb: f64,
    pub priority: isize,
}

#[derive(Debug, Clone, Default)]
pub struct SwapUpdate {
    #[cfg(target_os = "linux")]
    pub swap_devices: Vec<InfoSwap>,
    pub total_swap: u64,
    pub used_swap: u64,
}

#[derive(Debug, Clone, Default)]
pub enum SizeUnits {
    #[default]
    KB,
    MB,
    GB,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Error)]
pub enum SwapDataError {
    #[error("Procfs error: {0}")]
    Procfs(#[from] procfs::ProcError),
    #[error("I/O error accessing /proc: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(target_os = "windows")]
#[derive(Debug, Error)]
pub enum SwapDataError {
    #[error("I/O error accessing system information: {0}")]
    Io(#[from] io::Error),
}

#[cfg(target_os = "linux")]
pub fn get_swap_devices(unit: SizeUnits) -> std::io::Result<Vec<InfoSwap>> {
    let mut out = Vec::new();
    for swap in SwapIter::new()? {
        let s = swap?;
        out.push(InfoSwap {
            name: s.source.to_string_lossy().into_owned(),
            kind: s.kind.to_string_lossy().into_owned(),
            size_kb: s.size as u64,
            used_kb: convert_swap(s.used as u64, unit.to_owned()),
            priority: s.priority,
        });
    }
    Ok(out)
}

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
                                Err(_) => "unknown".to_string(),
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
            }
            Err(_) => {}
        }
    }

    Ok(swap_processes)
}

#[cfg(target_os = "windows")]
pub fn get_processes_using_swap(unit: SizeUnits) -> Result<Vec<ProcessSwapInfo>, SwapDataError> {
    let mut profile_page_processes = Vec::new();

    match tasklist::Tasklist::new() {
        Ok(tasks) => {
            for task in tasks {
                let meminfo = task.get_memory_info();

                let info = ProcessSwapInfo {
                    pid: task.pid,
                    name: task.pname,
                    swap_size: convert_swap(
                        meminfo.get_pagefile_usage() as u64 / 1024,
                        unit.clone(),
                    ),
                };
                profile_page_processes.push(info);
            }
        }
        Err(_) => {}
    }

    Ok(profile_page_processes)
}

#[cfg(target_os = "linux")]
pub fn get_chart_info(unit: SizeUnits) -> Result<SwapUpdate, SwapDataError> {
    let meminfo = Meminfo::current()?;

    let total_swap_kb = meminfo.swap_total / 1024;
    let used_swap_kb = meminfo.swap_total.saturating_sub(meminfo.swap_free) / 1024;
    let swap_devices = get_swap_devices(unit)?;

    Ok(SwapUpdate {
        swap_devices: swap_devices,
        total_swap: total_swap_kb,
        used_swap: used_swap_kb,
    })
}

#[cfg(target_os = "windows")]
pub fn get_chart_info() -> Result<SwapUpdate, SwapDataError> {
    use std::mem::MaybeUninit;
    use winapi::um::sysinfoapi::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

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
