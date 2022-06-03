use std::io;

use crate::sys_fs_read_separated;

use super::sys_fs_type;

sys_fs_type!(CPU, RO, u32, KernelMax, "kernel_max");
sys_fs_type!(CPU, RO, Vec<u8>, Offline, "offline");
sys_fs_type!(CPU, RO, Vec<u8>, Online, "online");
sys_fs_type!(CPU, RO, Vec<u8>, Possible, "possible");
sys_fs_type!(CPU, RO, Vec<u8>, Present, "present");
//cpu_subpath!(INTELPSTATE, "intel_pstate");
sys_fs_type!(CPU, RO, bool, Boost, "cpufreq/boost");

pub struct CpuDriver {
    kernel_max: KernelMax,
    offline: Offline,
    online: Online,
    possible: Possible,
    present: Present,
    boost: Boost,
}

impl CpuDriver {
    fn get_available_logical_cores(&mut self) -> Result<Vec<()>, io::Error> {
        // Add "possible" and "present" logical cores
        //let cores = Vec::new();
        let possible_cores = sys_fs_read_separated(&mut self.possible)?;
        let present_cores = sys_fs_read_separated(&mut self.present)?;
        let mut core_index_to_add: Vec<u8> = Vec::new();

        for possible_core_index in possible_cores {
            if present_cores.contains(&possible_core_index) {
                core_index_to_add.push(possible_core_index);
            }
        }

        core_index_to_add.sort_unstable();

        //    core_index_to_add.into_iter().map(|core_index| {
        //        let newCore = LogicalCpuController(this.basePath, core_index);
        //        if (core_index == 0 || newCore.online.isAvailable()) {
        //            this.cores.push(newCore);
        //        }
        //    })
        todo!();
    }
}
