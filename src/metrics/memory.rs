use super::history::History;
use sysinfo::System;

pub struct MemoryMetrics {
    pub total_ram: u64,
    pub used_ram: u64,
    pub app_memory: u64,
    pub wired: u64,
    pub compressed: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub ram_percent: f64,
    pub swap_percent: f64,
    pub ram_history: History,
    pub swap_history: History,
}

impl MemoryMetrics {
    pub fn new() -> Self {
        Self {
            total_ram: 0,
            used_ram: 0,
            app_memory: 0,
            wired: 0,
            compressed: 0,
            total_swap: 0,
            used_swap: 0,
            ram_percent: 0.0,
            swap_percent: 0.0,
            ram_history: History::new(),
            swap_history: History::new(),
        }
    }

    pub fn update(&mut self, sys: &System) {
        self.total_ram = sys.total_memory();
        self.total_swap = sys.total_swap();
        self.used_swap = sys.used_swap();

        // Use native macOS API for accurate memory usage matching Activity Monitor
        if let Some(vm) = get_vm_statistics() {
            let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
            let free_pages = (vm.free_count as u64).saturating_sub(vm.speculative_count as u64);
            let available = (free_pages + vm.external_page_count as u64) * page_size;
            self.used_ram = self.total_ram.saturating_sub(available);

            self.app_memory = (vm.internal_page_count as u64)
                .saturating_sub(vm.purgeable_count as u64)
                * page_size;
            self.wired = vm.wire_count as u64 * page_size;
            self.compressed = vm.compressor_page_count as u64 * page_size;
        } else {
            self.used_ram = sys.used_memory();
            self.app_memory = 0;
            self.wired = 0;
            self.compressed = 0;
        }

        self.ram_percent = if self.total_ram > 0 {
            (self.used_ram as f64 / self.total_ram as f64) * 100.0
        } else {
            0.0
        };

        self.swap_percent = if self.total_swap > 0 {
            (self.used_swap as f64 / self.total_swap as f64) * 100.0
        } else {
            0.0
        };

        self.ram_history.push(self.ram_percent);
        self.swap_history.push(self.swap_percent);
    }
}

// --- Native macOS VM statistics via host_statistics64 ---

const HOST_VM_INFO64: i32 = 4;
const HOST_VM_INFO64_COUNT: u32 = 38; // sizeof(vm_statistics64_data_t) / sizeof(integer_t)
const KERN_SUCCESS: i32 = 0;

#[repr(C)]
struct VmStatistics64 {
    free_count: u32,
    active_count: u32,
    inactive_count: u32,
    wire_count: u32,
    zero_fill_count: u64,
    reactivations: u64,
    pageins: u64,
    pageouts: u64,
    faults: u64,
    cow_faults: u64,
    lookups: u64,
    hits: u64,
    purges: u64,
    purgeable_count: u32,
    speculative_count: u32,
    decompressions: u64,
    compressions: u64,
    swapins: u64,
    swapouts: u64,
    compressor_page_count: u32,
    throttled_count: u32,
    external_page_count: u32,
    internal_page_count: u32,
    total_uncompressed_pages_in_compressor: u64,
}

extern "C" {
    fn mach_host_self() -> u32;
    fn host_statistics64(host: u32, flavor: i32, info: *mut i32, count: *mut u32) -> i32;
}

fn get_vm_statistics() -> Option<VmStatistics64> {
    unsafe {
        let mut stat: VmStatistics64 = std::mem::zeroed();
        let mut count = HOST_VM_INFO64_COUNT;
        let ret = host_statistics64(
            mach_host_self(),
            HOST_VM_INFO64,
            &mut stat as *mut VmStatistics64 as *mut i32,
            &mut count,
        );
        if ret == KERN_SUCCESS {
            Some(stat)
        } else {
            None
        }
    }
}
