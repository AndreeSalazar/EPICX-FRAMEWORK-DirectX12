//! Abstract Syntax Tree for .gpu language

/// A complete .gpu program
#[derive(Debug, Clone, Default)]
pub struct Program {
    pub shaders: Vec<ShaderDecl>,
    pub buffers: Vec<BufferDecl>,
    pub textures: Vec<TextureDecl>,
    pub pipelines: Vec<PipelineDecl>,
    pub compute_pipelines: Vec<ComputeDecl>,
    pub frames: Vec<FrameDecl>,
    pub queues: Vec<QueueDecl>,
}

/// Shader declaration
#[derive(Debug, Clone)]
pub struct ShaderDecl {
    pub name: String,
    pub path: String,
    pub shader_type: ShaderType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Vertex,
    Pixel,
    Compute,
    Geometry,
    Hull,
    Domain,
}

/// Buffer declaration
#[derive(Debug, Clone)]
pub struct BufferDecl {
    pub name: String,
    pub element_type: ElementType,
    pub count: u32,
    pub heap_type: HeapType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    F32,
    F32x2,
    F32x3,
    F32x4,
    U32,
    I32,
    U16,
    Mat4,
}

impl ElementType {
    pub fn size_bytes(&self) -> u32 {
        match self {
            ElementType::F32 => 4,
            ElementType::F32x2 => 8,
            ElementType::F32x3 => 12,
            ElementType::F32x4 => 16,
            ElementType::U32 => 4,
            ElementType::I32 => 4,
            ElementType::U16 => 2,
            ElementType::Mat4 => 64,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeapType {
    #[default]
    Default,
    Upload,
    Readback,
}

/// Texture declaration
#[derive(Debug, Clone)]
pub struct TextureDecl {
    pub name: String,
    pub format: TextureFormat,
    pub width: u32,
    pub height: u32,
    pub heap_type: HeapType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextureFormat {
    #[default]
    RGBA8,
    RGBA16F,
    RGBA32F,
    R8,
    R16F,
    R32F,
    Depth24Stencil8,
    Depth32F,
}

/// Graphics pipeline declaration
#[derive(Debug, Clone, Default)]
pub struct PipelineDecl {
    pub name: String,
    pub vertex_shader: Option<String>,
    pub pixel_shader: Option<String>,
    pub geometry_shader: Option<String>,
    pub topology: Topology,
    pub cull_mode: CullMode,
    pub depth_enabled: bool,
    pub blend_mode: BlendMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Topology {
    #[default]
    Triangles,
    TriangleStrip,
    Lines,
    LineStrip,
    Points,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CullMode {
    None,
    Front,
    #[default]
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    None,
    Alpha,
    Additive,
    Multiply,
}

/// Compute pipeline declaration
#[derive(Debug, Clone)]
pub struct ComputeDecl {
    pub name: String,
    pub shader: String,
    pub threads_x: u32,
    pub threads_y: u32,
    pub threads_z: u32,
}

/// Frame declaration (executed each frame)
#[derive(Debug, Clone)]
pub struct FrameDecl {
    pub name: String,
    pub commands: Vec<Command>,
}

/// Queue declaration (for multi-queue)
#[derive(Debug, Clone)]
pub struct QueueDecl {
    pub name: String,
    pub queue_type: QueueType,
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueueType {
    #[default]
    Graphics,
    Compute,
    Copy,
}

/// GPU Commands
#[derive(Debug, Clone)]
pub enum Command {
    // Clear commands
    ClearColor { r: f32, g: f32, b: f32, a: f32 },
    ClearDepth { depth: f32 },
    
    // State commands
    Viewport { x: u32, y: u32, width: u32, height: u32 },
    Scissor { x: u32, y: u32, width: u32, height: u32 },
    UsePipeline { name: String },
    UseCompute { name: String },
    
    // Bind commands
    BindBuffer { buffer: String, slot: u32, stride: u32 },
    BindTexture { texture: String, slot: u32 },
    BindConstant { buffer: String, slot: u32 },
    
    // Draw commands
    Draw { vertex_count: u32 },
    DrawIndexed { index_count: u32 },
    DrawInstanced { vertex_count: u32, instance_count: u32 },
    
    // Compute commands
    Dispatch { x: u32, y: u32, z: u32 },
    
    // Sync commands
    Barrier,
    Wait { queue: String },
    Signal { queue: String },
    
    // Present
    Present,
}

impl Program {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Get statistics about the program
    pub fn stats(&self) -> ProgramStats {
        let mut total_commands = 0;
        for frame in &self.frames {
            total_commands += frame.commands.len();
        }
        for queue in &self.queues {
            total_commands += queue.commands.len();
        }
        
        ProgramStats {
            shader_count: self.shaders.len(),
            buffer_count: self.buffers.len(),
            texture_count: self.textures.len(),
            pipeline_count: self.pipelines.len(),
            compute_count: self.compute_pipelines.len(),
            frame_count: self.frames.len(),
            queue_count: self.queues.len(),
            total_commands,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramStats {
    pub shader_count: usize,
    pub buffer_count: usize,
    pub texture_count: usize,
    pub pipeline_count: usize,
    pub compute_count: usize,
    pub frame_count: usize,
    pub queue_count: usize,
    pub total_commands: usize,
}
