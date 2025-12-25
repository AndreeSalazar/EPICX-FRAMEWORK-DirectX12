//! GPU Detection and Information
//!
//! Robust GPU detection system that analyzes all available adapters
//! and selects the best one for rendering.

use windows::Win32::Graphics::{
    Direct3D::D3D_FEATURE_LEVEL_12_0,
    Direct3D12::*,
    Dxgi::*,
};

/// GPU vendor identification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Microsoft,  // WARP software renderer
    Unknown,
}

impl GpuVendor {
    pub fn from_vendor_id(id: u32) -> Self {
        match id {
            0x10DE => GpuVendor::Nvidia,
            0x1002 | 0x1022 => GpuVendor::Amd,
            0x8086 => GpuVendor::Intel,
            0x1414 => GpuVendor::Microsoft,
            _ => GpuVendor::Unknown,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            GpuVendor::Nvidia => "NVIDIA",
            GpuVendor::Amd => "AMD",
            GpuVendor::Intel => "Intel",
            GpuVendor::Microsoft => "Microsoft (WARP)",
            GpuVendor::Unknown => "Unknown",
        }
    }
}

/// Information about a GPU adapter
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub vendor_id: u32,
    pub device_id: u32,
    pub dedicated_video_memory: u64,
    pub dedicated_system_memory: u64,
    pub shared_system_memory: u64,
    pub is_software: bool,
    pub supports_dx12: bool,
    pub adapter_index: u32,
    pub score: u32,  // Higher = better
}

impl GpuInfo {
    /// Calculate a score for GPU selection (higher is better)
    fn calculate_score(&mut self) {
        let mut score = 0u32;
        
        // Prefer hardware over software
        if !self.is_software {
            score += 10000;
        }
        
        // Prefer discrete GPUs (more VRAM)
        score += (self.dedicated_video_memory / (1024 * 1024 * 100)) as u32; // +1 per 100MB VRAM
        
        // Vendor preference (discrete GPUs typically)
        match self.vendor {
            GpuVendor::Nvidia => score += 500,
            GpuVendor::Amd => score += 500,
            GpuVendor::Intel => score += 100,  // Often integrated
            GpuVendor::Microsoft => score += 0, // WARP
            GpuVendor::Unknown => score += 50,
        }
        
        // Must support DX12
        if !self.supports_dx12 {
            score = 0;
        }
        
        self.score = score;
    }
    
    /// Format memory size for display
    pub fn format_memory(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{} KB", bytes / 1024)
        }
    }
}

impl std::fmt::Display for GpuInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) - VRAM: {}, DX12: {}",
            self.name,
            self.vendor.name(),
            Self::format_memory(self.dedicated_video_memory),
            if self.supports_dx12 { "Yes" } else { "No" }
        )
    }
}

/// GPU detection and selection system
pub struct GpuDetector {
    gpus: Vec<GpuInfo>,
    selected_index: Option<usize>,
}

impl GpuDetector {
    /// Detect all available GPUs
    pub fn detect() -> Self {
        let mut detector = Self {
            gpus: Vec::new(),
            selected_index: None,
        };
        
        unsafe {
            // Create DXGI factory
            let factory: Result<IDXGIFactory4, _> = CreateDXGIFactory2(DXGI_CREATE_FACTORY_FLAGS(0));
            let Ok(factory) = factory else {
                eprintln!("[GPU] Failed to create DXGI factory");
                return detector;
            };
            
            // Enumerate all adapters
            let mut adapter_index = 0u32;
            loop {
                let adapter_result = factory.EnumAdapters1(adapter_index);
                match adapter_result {
                    Ok(adapter) => {
                        if let Ok(desc) = adapter.GetDesc1() {
                            let name = String::from_utf16_lossy(
                                &desc.Description[..desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len())]
                            );
                            
                            let is_software = (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0;
                            
                            // Check DX12 support
                            let supports_dx12 = D3D12CreateDevice(
                                &adapter,
                                D3D_FEATURE_LEVEL_12_0,
                                std::ptr::null_mut::<Option<ID3D12Device>>(),
                            ).is_ok();
                            
                            let mut gpu_info = GpuInfo {
                                name,
                                vendor: GpuVendor::from_vendor_id(desc.VendorId),
                                vendor_id: desc.VendorId,
                                device_id: desc.DeviceId,
                                dedicated_video_memory: desc.DedicatedVideoMemory as u64,
                                dedicated_system_memory: desc.DedicatedSystemMemory as u64,
                                shared_system_memory: desc.SharedSystemMemory as u64,
                                is_software,
                                supports_dx12,
                                adapter_index,
                                score: 0,
                            };
                            
                            gpu_info.calculate_score();
                            detector.gpus.push(gpu_info);
                        }
                        adapter_index += 1;
                    }
                    Err(_) => break,
                }
            }
        }
        
        // Sort by score (best first)
        detector.gpus.sort_by(|a, b| b.score.cmp(&a.score));
        
        // Select best GPU
        if !detector.gpus.is_empty() {
            detector.selected_index = Some(0);
        }
        
        detector
    }
    
    /// Get all detected GPUs
    pub fn all_gpus(&self) -> &[GpuInfo] {
        &self.gpus
    }
    
    /// Get the best (selected) GPU
    pub fn best_gpu(&self) -> Option<&GpuInfo> {
        self.selected_index.map(|i| &self.gpus[i])
    }
    
    /// Get the selected GPU index
    pub fn selected_adapter_index(&self) -> Option<u32> {
        self.best_gpu().map(|g| g.adapter_index)
    }
    
    /// Check if any GPU was found
    pub fn has_gpu(&self) -> bool {
        self.gpus.iter().any(|g| g.supports_dx12 && !g.is_software)
    }
    
    /// Print GPU information
    pub fn print_info(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                    GPU DETECTION REPORT                      ║");
        println!("╠══════════════════════════════════════════════════════════════╣");
        
        if self.gpus.is_empty() {
            println!("║  No GPUs detected!                                           ║");
        } else {
            for (i, gpu) in self.gpus.iter().enumerate() {
                let selected = if self.selected_index == Some(i) { "→" } else { " " };
                let status = if gpu.supports_dx12 { "✓" } else { "✗" };
                
                println!("║ {} [{}] {}",
                    selected,
                    status,
                    truncate_str(&gpu.name, 54)
                );
                println!("║      Vendor: {} | VRAM: {}",
                    gpu.vendor.name(),
                    GpuInfo::format_memory(gpu.dedicated_video_memory)
                );
                println!("║      Score: {} | Software: {}",
                    gpu.score,
                    if gpu.is_software { "Yes" } else { "No" }
                );
                
                if i < self.gpus.len() - 1 {
                    println!("║ ──────────────────────────────────────────────────────────── ║");
                }
            }
        }
        
        println!("╚══════════════════════════════════════════════════════════════╝");
        
        if let Some(gpu) = self.best_gpu() {
            println!("[GPU] Selected: {} ({})", gpu.name, gpu.vendor.name());
        }
    }
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:<width$}", s, width = max_len)
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Quick function to detect and print GPU info
pub fn detect_gpu() -> Option<GpuInfo> {
    let detector = GpuDetector::detect();
    detector.print_info();
    detector.best_gpu().cloned()
}
