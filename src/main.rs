mod swap_info;

use swap_info::{get_processes_using_swap, ProcessSwapInfo, SwapDataError};

fn main() {
   match get_processes_using_swap(){
   	Ok(sp) => {
   		for ps in sp {
   			println!("{} {} {}", ps.pid, ps.name, ps.swap_kb);
   		}
   	}
   	Err(e) => {
   		println!("{}", e);
   	}
   }
}
