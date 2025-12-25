//! Frame management for double/triple buffering

use crate::dx12::{CommandAllocator, CommandList, Device, Dx12Result};

/// Per-frame resources for multi-buffering
pub struct FrameResources {
    pub command_allocator: CommandAllocator,
    pub fence_value: u64,
}

impl FrameResources {
    /// Create new frame resources
    pub fn new(device: &Device) -> Dx12Result<Self> {
        let command_allocator = CommandAllocator::new(
            device,
            windows::Win32::Graphics::Direct3D12::D3D12_COMMAND_LIST_TYPE_DIRECT,
        )?;

        Ok(Self {
            command_allocator,
            fence_value: 0,
        })
    }

    /// Reset the frame resources for reuse
    pub fn reset(&mut self) -> Dx12Result<()> {
        self.command_allocator.reset()
    }
}

/// Represents a single frame being rendered
pub struct Frame {
    index: u32,
    resources: FrameResources,
}

impl Frame {
    /// Create a new frame
    pub fn new(index: u32, device: &Device) -> Dx12Result<Self> {
        Ok(Self {
            index,
            resources: FrameResources::new(device)?,
        })
    }

    /// Get the frame index
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Get the frame resources
    pub fn resources(&self) -> &FrameResources {
        &self.resources
    }

    /// Get mutable frame resources
    pub fn resources_mut(&mut self) -> &mut FrameResources {
        &mut self.resources
    }

    /// Begin recording commands for this frame
    pub fn begin(&mut self) -> Dx12Result<()> {
        self.resources.reset()
    }

    /// End recording commands for this frame
    pub fn end(&mut self) -> Dx12Result<()> {
        Ok(())
    }
}

/// Frame manager for handling multiple frames in flight
pub struct FrameManager {
    frames: Vec<Frame>,
    current_frame: usize,
    frame_count: u32,
}

impl FrameManager {
    /// Create a new frame manager
    pub fn new(device: &Device, frame_count: u32) -> Dx12Result<Self> {
        let mut frames = Vec::with_capacity(frame_count as usize);
        for i in 0..frame_count {
            frames.push(Frame::new(i, device)?);
        }

        Ok(Self {
            frames,
            current_frame: 0,
            frame_count,
        })
    }

    /// Get the current frame
    pub fn current(&self) -> &Frame {
        &self.frames[self.current_frame]
    }

    /// Get the current frame mutably
    pub fn current_mut(&mut self) -> &mut Frame {
        &mut self.frames[self.current_frame]
    }

    /// Advance to the next frame
    pub fn next_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frame_count as usize;
    }

    /// Get the frame count
    pub fn frame_count(&self) -> u32 {
        self.frame_count
    }
}
