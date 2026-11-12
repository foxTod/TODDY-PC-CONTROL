use std::time::{Duration, Instant};
use sysinfo::{ProcessesToUpdate, System};

pub struct PcMonitor {
    system: System,
    last_refresh: Instant,
}

impl PcMonitor {
    pub fn new() -> Result<Self, sysinfo::Error> {
        let mut system = System::new_all()?;
        system.refresh_cpu_usage();

        Ok(Self {
            system,
            last_refresh: Instant::now(),
        })
    }

    pub fn refresh(&mut self) {
        if self.last_refresh.elapsed() >= Duration::from_secs(1) {
            self.system.refresh_cpu_usage();
            self.system.refresh_memory();
            self.system.refresh_processes(ProcessesToUpdate::All, true);
            self.last_refresh = Instant::now();
        }
    }

    pub fn cpu_usage(&self) -> f32 {
        self.system.global_cpu_usage()
    }

    pub fn used_memory(&self) -> u64 {
        self.system.used_memory()
    }

    pub fn total_memory(&self) -> u64 {
        self.system.total_memory()
    }

    pub fn process_count(&self) -> usize {
        self.system.processes().len()
    }

    pub fn uptime_seconds(&self) -> Result<u64, sysinfo::Error> {
        System::uptime()
    }
}
