//! Command Queue and Command List wrappers

use super::{Device, Dx12Result, Fence};
use windows::core::Interface;
use windows::Win32::Graphics::Direct3D12::*;

/// Command queue wrapper
pub struct CommandQueue {
    queue: ID3D12CommandQueue,
    queue_type: D3D12_COMMAND_LIST_TYPE,
    fence: Fence,
    fence_value: u64,
}

impl CommandQueue {
    /// Create a new command queue
    pub fn new(device: &Device, queue_type: D3D12_COMMAND_LIST_TYPE) -> Dx12Result<Self> {
        let queue = device.create_command_queue(queue_type)?;
        let fence = Fence::new(device, 0)?;

        Ok(Self {
            queue,
            queue_type,
            fence,
            fence_value: 0,
        })
    }

    /// Create a graphics command queue
    pub fn graphics(device: &Device) -> Dx12Result<Self> {
        Self::new(device, D3D12_COMMAND_LIST_TYPE_DIRECT)
    }

    /// Create a compute command queue
    pub fn compute(device: &Device) -> Dx12Result<Self> {
        Self::new(device, D3D12_COMMAND_LIST_TYPE_COMPUTE)
    }

    /// Create a copy command queue
    pub fn copy(device: &Device) -> Dx12Result<Self> {
        Self::new(device, D3D12_COMMAND_LIST_TYPE_COPY)
    }

    /// Get the raw queue handle
    pub fn raw(&self) -> &ID3D12CommandQueue {
        &self.queue
    }

    /// Get the queue type
    pub fn queue_type(&self) -> D3D12_COMMAND_LIST_TYPE {
        self.queue_type
    }

    /// Execute command lists
    pub fn execute(&self, command_lists: &[&CommandList]) {
        unsafe {
            let lists: Vec<Option<ID3D12CommandList>> = command_lists
                .iter()
                .filter_map(|cl| cl.raw().cast::<ID3D12CommandList>().ok())
                .map(Some)
                .collect();
            self.queue.ExecuteCommandLists(&lists);
        }
    }

    /// Signal the fence
    pub fn signal(&mut self) -> Dx12Result<u64> {
        self.fence_value += 1;
        self.fence.signal(&self.queue, self.fence_value)?;
        Ok(self.fence_value)
    }

    /// Wait for the fence to reach a value
    pub fn wait_for_fence(&self, value: u64) -> Dx12Result<()> {
        self.fence.wait(value)
    }

    /// Flush all pending commands
    pub fn flush(&mut self) -> Dx12Result<()> {
        let value = self.signal()?;
        self.wait_for_fence(value)
    }
}

/// Command allocator wrapper
pub struct CommandAllocator {
    allocator: ID3D12CommandAllocator,
    list_type: D3D12_COMMAND_LIST_TYPE,
}

impl CommandAllocator {
    /// Create a new command allocator
    pub fn new(device: &Device, list_type: D3D12_COMMAND_LIST_TYPE) -> Dx12Result<Self> {
        let allocator = device.create_command_allocator(list_type)?;
        Ok(Self { allocator, list_type })
    }

    /// Get the raw allocator handle
    pub fn raw(&self) -> &ID3D12CommandAllocator {
        &self.allocator
    }

    /// Reset the allocator
    pub fn reset(&self) -> Dx12Result<()> {
        unsafe {
            self.allocator.Reset()?;
            Ok(())
        }
    }
}

/// Command list wrapper
pub struct CommandList {
    list: ID3D12GraphicsCommandList,
    list_type: D3D12_COMMAND_LIST_TYPE,
}

impl CommandList {
    /// Create a new command list
    pub fn new(
        device: &Device,
        allocator: &CommandAllocator,
        initial_state: Option<&ID3D12PipelineState>,
    ) -> Dx12Result<Self> {
        unsafe {
            let list: ID3D12GraphicsCommandList = device.raw().CreateCommandList(
                0,
                allocator.list_type,
                allocator.raw(),
                initial_state,
            )?;

            Ok(Self {
                list,
                list_type: allocator.list_type,
            })
        }
    }

    /// Get the raw command list handle
    pub fn raw(&self) -> &ID3D12GraphicsCommandList {
        &self.list
    }

    /// Close the command list
    pub fn close(&self) -> Dx12Result<()> {
        unsafe {
            self.list.Close()?;
            Ok(())
        }
    }

    /// Reset the command list
    pub fn reset(
        &self,
        allocator: &CommandAllocator,
        initial_state: Option<&ID3D12PipelineState>,
    ) -> Dx12Result<()> {
        unsafe {
            self.list.Reset(allocator.raw(), initial_state)?;
            Ok(())
        }
    }

    /// Set the viewport
    pub fn set_viewport(&self, x: f32, y: f32, width: f32, height: f32) {
        unsafe {
            let viewport = D3D12_VIEWPORT {
                TopLeftX: x,
                TopLeftY: y,
                Width: width,
                Height: height,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };
            self.list.RSSetViewports(&[viewport]);
        }
    }

    /// Set the scissor rect
    pub fn set_scissor_rect(&self, left: i32, top: i32, right: i32, bottom: i32) {
        unsafe {
            let rect = windows::Win32::Foundation::RECT {
                left,
                top,
                right,
                bottom,
            };
            self.list.RSSetScissorRects(&[rect]);
        }
    }

    /// Set render targets
    pub fn set_render_targets(
        &self,
        rtv_handles: &[D3D12_CPU_DESCRIPTOR_HANDLE],
        dsv_handle: Option<*const D3D12_CPU_DESCRIPTOR_HANDLE>,
    ) {
        unsafe {
            self.list.OMSetRenderTargets(
                rtv_handles.len() as u32,
                Some(rtv_handles.as_ptr()),
                false,
                dsv_handle,
            );
        }
    }

    /// Clear a render target
    pub fn clear_render_target(
        &self,
        rtv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,
        color: [f32; 4],
    ) {
        unsafe {
            self.list.ClearRenderTargetView(rtv_handle, &color, None);
        }
    }

    /// Clear a depth stencil
    pub fn clear_depth_stencil(
        &self,
        dsv_handle: D3D12_CPU_DESCRIPTOR_HANDLE,
        depth: f32,
        stencil: u8,
    ) {
        unsafe {
            self.list.ClearDepthStencilView(
                dsv_handle,
                D3D12_CLEAR_FLAG_DEPTH | D3D12_CLEAR_FLAG_STENCIL,
                depth,
                stencil,
                &[],
            );
        }
    }

    /// Set primitive topology
    pub fn set_primitive_topology(&self, topology: D3D_PRIMITIVE_TOPOLOGY) {
        unsafe {
            self.list.IASetPrimitiveTopology(topology);
        }
    }

    /// Set vertex buffers
    pub fn set_vertex_buffers(&self, start_slot: u32, views: &[D3D12_VERTEX_BUFFER_VIEW]) {
        unsafe {
            self.list.IASetVertexBuffers(start_slot, Some(views));
        }
    }

    /// Set index buffer
    pub fn set_index_buffer(&self, view: &D3D12_INDEX_BUFFER_VIEW) {
        unsafe {
            self.list.IASetIndexBuffer(Some(view));
        }
    }

    /// Draw instanced
    pub fn draw_instanced(
        &self,
        vertex_count: u32,
        instance_count: u32,
        start_vertex: u32,
        start_instance: u32,
    ) {
        unsafe {
            self.list.DrawInstanced(vertex_count, instance_count, start_vertex, start_instance);
        }
    }

    /// Draw indexed instanced
    pub fn draw_indexed_instanced(
        &self,
        index_count: u32,
        instance_count: u32,
        start_index: u32,
        base_vertex: i32,
        start_instance: u32,
    ) {
        unsafe {
            self.list.DrawIndexedInstanced(
                index_count,
                instance_count,
                start_index,
                base_vertex,
                start_instance,
            );
        }
    }

    /// Resource barrier
    pub fn resource_barrier(&self, barriers: &[D3D12_RESOURCE_BARRIER]) {
        unsafe {
            self.list.ResourceBarrier(barriers);
        }
    }
}

use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY;
