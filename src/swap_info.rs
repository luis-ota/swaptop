use thiserror::Error;
use procfs;

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
}

#[derive(Debug, Error)]
pub enum SwapDataError {
    #[error("Procfs error: {0}")]
    Procfs(#[from] procfs::ProcError),
    #[error("I/O error accessing /proc: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]

pub struct AggregatedSwapInfo {
    pub name: String,           
    pub total_swap_kb: u64,     
    pub pids: Vec<i32>,        
    pub process_count: usize, 
}


pub fn get_processes_using_swap() -> Result<Vec<ProcessSwapInfo>, SwapDataError> {
    let mut swap_processes = Vec::new();

    for process_result in procfs::process::all_processes()? {
        match process_result {
            Ok(process) => {
            	let pid = process.pid;
            	match process.status(){
            		Ok(status) => {
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
            		Err(_) => {}
            	}
            },
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
    
    Ok(swap_processes)
}

// pub fn get_aggregated_swap_info() -> Result<Vec<AggregatedSwapInfo>, SwapDataError>{
// 	let aggregated_swap_ps = Vec::new();
// 	
// 	match get_aggregated_swap_info(){
// 		Ok(processes) => {
// 		let mapped_ps = HashMap::new();
// 
// 		
// 			
// 		}
// 		Err(e) => {
// 			println!("{}", e);
// 		}
// 	}
// }
