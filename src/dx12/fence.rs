//! Fence wrapper for GPU synchronization

use super::{Device, Dx12Result};
use windows::Win32::{
    Foundation::HANDLE,
    Graphics::Direct3D12::*,
    System::Threading::{CreateEventW, WaitForSingleObject, INFINITE},
};

/// Fence wrapper
pub struct Fence {
    fence: ID3D12Fence,
    event: HANDLE,
}

impl Fence {
    /// Create a new fence
    pub fn new(device: &Device, initial_value: u64) -> Dx12Result<Self> {
        let fence = device.create_fence(initial_value)?;
        
        unsafe {
            let event = CreateEventW(None, false, false, None)?;
            Ok(Self { fence, event })
        }
    }

    /// Get the raw fence
    pub fn raw(&self) -> &ID3D12Fence {
        &self.fence
    }

    /// Get the current completed value
    pub fn completed_value(&self) -> u64 {
        unsafe { self.fence.GetCompletedValue() }
    }

    /// Signal the fence from the GPU
    pub fn signal(&self, queue: &ID3D12CommandQueue, value: u64) -> Dx12Result<()> {
        unsafe {
            queue.Signal(&self.fence, value)?;
            Ok(())
        }
    }

    /// Wait for the fence to reach a value
    pub fn wait(&self, value: u64) -> Dx12Result<()> {
        unsafe {
            if self.fence.GetCompletedValue() < value {
                self.fence.SetEventOnCompletion(value, self.event)?;
                WaitForSingleObject(self.event, INFINITE);
            }
            Ok(())
        }
    }

    /// Wait with a timeout (returns true if completed)
    pub fn wait_timeout(&self, value: u64, timeout_ms: u32) -> Dx12Result<bool> {
        unsafe {
            if self.fence.GetCompletedValue() < value {
                self.fence.SetEventOnCompletion(value, self.event)?;
                let result = WaitForSingleObject(self.event, timeout_ms);
                return Ok(result.0 == 0); // WAIT_OBJECT_0
            }
            Ok(true)
        }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            if !self.event.is_invalid() {
                windows::Win32::Foundation::CloseHandle(self.event).ok();
            }
        }
    }
}
