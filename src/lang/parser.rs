//! Parser for .gpu language

use super::ast::*;
use super::lexer::{Token, TokenKind};
use super::LangError;

/// Parse error
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at line {}: {}", self.line, self.message)
    }
}

/// Parser for .gpu source
pub struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, position: 0 }
    }
    
    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    
    fn peek_kind(&self) -> Option<&TokenKind> {
        self.current().map(|t| &t.kind)
    }
    
    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }
    
    fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), Some(TokenKind::Newline)) {
            self.advance();
        }
    }
    
    fn expect_identifier(&mut self) -> Result<String, LangError> {
        match self.advance() {
            Some(Token { kind: TokenKind::Identifier(s), .. }) => Ok(s.clone()),
            Some(t) => Err(LangError::Parser {
                line: t.line,
                message: format!("Expected identifier, got {:?}", t.kind),
            }),
            None => Err(LangError::Parser {
                line: 0,
                message: "Unexpected end of input".to_string(),
            }),
        }
    }
    
    fn expect_string(&mut self) -> Result<String, LangError> {
        match self.advance() {
            Some(Token { kind: TokenKind::String(s), .. }) => Ok(s.clone()),
            Some(t) => Err(LangError::Parser {
                line: t.line,
                message: format!("Expected string, got {:?}", t.kind),
            }),
            None => Err(LangError::Parser {
                line: 0,
                message: "Unexpected end of input".to_string(),
            }),
        }
    }
    
    fn expect_integer(&mut self) -> Result<i64, LangError> {
        match self.advance() {
            Some(Token { kind: TokenKind::Integer(n), .. }) => Ok(*n),
            Some(t) => Err(LangError::Parser {
                line: t.line,
                message: format!("Expected integer, got {:?}", t.kind),
            }),
            None => Err(LangError::Parser {
                line: 0,
                message: "Unexpected end of input".to_string(),
            }),
        }
    }
    
    fn expect_float(&mut self) -> Result<f64, LangError> {
        match self.advance() {
            Some(Token { kind: TokenKind::Float(n), .. }) => Ok(*n),
            Some(Token { kind: TokenKind::Integer(n), .. }) => Ok(*n as f64),
            Some(t) => Err(LangError::Parser {
                line: t.line,
                message: format!("Expected number, got {:?}", t.kind),
            }),
            None => Err(LangError::Parser {
                line: 0,
                message: "Unexpected end of input".to_string(),
            }),
        }
    }
    
    /// Parse a complete program
    pub fn parse_program(&mut self) -> Result<Program, LangError> {
        let mut program = Program::new();
        
        self.skip_newlines();
        
        while let Some(token) = self.current() {
            match &token.kind {
                TokenKind::Shader => {
                    self.advance();
                    program.shaders.push(self.parse_shader()?);
                }
                TokenKind::Buffer => {
                    self.advance();
                    program.buffers.push(self.parse_buffer()?);
                }
                TokenKind::Texture => {
                    self.advance();
                    program.textures.push(self.parse_texture()?);
                }
                TokenKind::Pipeline => {
                    self.advance();
                    program.pipelines.push(self.parse_pipeline()?);
                }
                TokenKind::Compute => {
                    self.advance();
                    program.compute_pipelines.push(self.parse_compute()?);
                }
                TokenKind::Frame => {
                    self.advance();
                    program.frames.push(self.parse_frame()?);
                }
                TokenKind::Queue => {
                    self.advance();
                    program.queues.push(self.parse_queue()?);
                }
                TokenKind::Newline => {
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
        
        Ok(program)
    }
    
    fn parse_shader(&mut self) -> Result<ShaderDecl, LangError> {
        let name = self.expect_identifier()?;
        let path = self.expect_string()?;
        
        // Infer shader type from name
        let shader_type = if name.ends_with("vs") || name.contains("vertex") {
            ShaderType::Vertex
        } else if name.ends_with("ps") || name.contains("pixel") {
            ShaderType::Pixel
        } else if name.ends_with("cs") || name.contains("compute") {
            ShaderType::Compute
        } else {
            ShaderType::Vertex
        };
        
        Ok(ShaderDecl { name, path, shader_type })
    }
    
    fn parse_buffer(&mut self) -> Result<BufferDecl, LangError> {
        let name = self.expect_identifier()?;
        
        let element_type = match self.peek_kind() {
            Some(TokenKind::F32) => { self.advance(); ElementType::F32 }
            Some(TokenKind::F32x2) => { self.advance(); ElementType::F32x2 }
            Some(TokenKind::F32x3) => { self.advance(); ElementType::F32x3 }
            Some(TokenKind::F32x4) => { self.advance(); ElementType::F32x4 }
            Some(TokenKind::U32) => { self.advance(); ElementType::U32 }
            Some(TokenKind::I32) => { self.advance(); ElementType::I32 }
            Some(TokenKind::U16) => { self.advance(); ElementType::U16 }
            Some(TokenKind::Mat4) => { self.advance(); ElementType::Mat4 }
            _ => ElementType::F32,
        };
        
        let count = self.expect_integer()? as u32;
        
        let heap_type = match self.peek_kind() {
            Some(TokenKind::Upload) => { self.advance(); HeapType::Upload }
            Some(TokenKind::Readback) => { self.advance(); HeapType::Readback }
            Some(TokenKind::Default) => { self.advance(); HeapType::Default }
            _ => HeapType::Default,
        };
        
        Ok(BufferDecl { name, element_type, count, heap_type })
    }
    
    fn parse_texture(&mut self) -> Result<TextureDecl, LangError> {
        let name = self.expect_identifier()?;
        
        let format = match self.peek_kind() {
            Some(TokenKind::RGBA8) => { self.advance(); TextureFormat::RGBA8 }
            Some(TokenKind::RGBA16F) => { self.advance(); TextureFormat::RGBA16F }
            Some(TokenKind::RGBA32F) => { self.advance(); TextureFormat::RGBA32F }
            _ => TextureFormat::RGBA8,
        };
        
        let width = self.expect_integer()? as u32;
        let height = self.expect_integer()? as u32;
        
        let heap_type = match self.peek_kind() {
            Some(TokenKind::Upload) => { self.advance(); HeapType::Upload }
            Some(TokenKind::Readback) => { self.advance(); HeapType::Readback }
            Some(TokenKind::Default) => { self.advance(); HeapType::Default }
            _ => HeapType::Default,
        };
        
        Ok(TextureDecl { name, format, width, height, heap_type })
    }
    
    fn parse_pipeline(&mut self) -> Result<PipelineDecl, LangError> {
        let name = self.expect_identifier()?;
        
        // Expect colon
        if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            self.advance();
        }
        
        let mut pipeline = PipelineDecl {
            name,
            ..Default::default()
        };
        
        self.skip_newlines();
        
        // Parse pipeline body
        while let Some(token) = self.current() {
            match &token.kind {
                TokenKind::Vertex => {
                    self.advance();
                    pipeline.vertex_shader = Some(self.expect_identifier()?);
                }
                TokenKind::Pixel => {
                    self.advance();
                    pipeline.pixel_shader = Some(self.expect_identifier()?);
                }
                TokenKind::Topology => {
                    self.advance();
                    pipeline.topology = match self.peek_kind() {
                        Some(TokenKind::Triangles) => { self.advance(); Topology::Triangles }
                        Some(TokenKind::TriangleStrip) => { self.advance(); Topology::TriangleStrip }
                        Some(TokenKind::Lines) => { self.advance(); Topology::Lines }
                        Some(TokenKind::Points) => { self.advance(); Topology::Points }
                        _ => Topology::Triangles,
                    };
                }
                TokenKind::Cull => {
                    self.advance();
                    pipeline.cull_mode = match self.peek_kind() {
                        Some(TokenKind::None) => { self.advance(); CullMode::None }
                        Some(TokenKind::Front) => { self.advance(); CullMode::Front }
                        Some(TokenKind::Back) => { self.advance(); CullMode::Back }
                        _ => CullMode::Back,
                    };
                }
                TokenKind::Depth => {
                    self.advance();
                    pipeline.depth_enabled = match self.peek_kind() {
                        Some(TokenKind::On) => { self.advance(); true }
                        Some(TokenKind::Off) => { self.advance(); false }
                        _ => true,
                    };
                }
                TokenKind::Blend => {
                    self.advance();
                    pipeline.blend_mode = match self.peek_kind() {
                        Some(TokenKind::None) => { self.advance(); BlendMode::None }
                        Some(TokenKind::Alpha) => { self.advance(); BlendMode::Alpha }
                        Some(TokenKind::Additive) => { self.advance(); BlendMode::Additive }
                        Some(TokenKind::Multiply) => { self.advance(); BlendMode::Multiply }
                        _ => BlendMode::None,
                    };
                }
                TokenKind::Newline => {
                    self.advance();
                }
                // End of pipeline block
                TokenKind::Shader | TokenKind::Buffer | TokenKind::Texture |
                TokenKind::Pipeline | TokenKind::Compute | TokenKind::Frame | TokenKind::Queue => {
                    break;
                }
                _ => {
                    self.advance();
                    break;
                }
            }
        }
        
        Ok(pipeline)
    }
    
    fn parse_compute(&mut self) -> Result<ComputeDecl, LangError> {
        let name = self.expect_identifier()?;
        
        if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            self.advance();
        }
        
        self.skip_newlines();
        
        let mut shader = String::new();
        let mut threads = (1u32, 1u32, 1u32);
        
        while let Some(token) = self.current() {
            match &token.kind {
                TokenKind::Shader => {
                    self.advance();
                    shader = self.expect_identifier()?;
                }
                TokenKind::Threads => {
                    self.advance();
                    threads.0 = self.expect_integer()? as u32;
                    threads.1 = self.expect_integer()? as u32;
                    threads.2 = self.expect_integer()? as u32;
                }
                TokenKind::Newline => {
                    self.advance();
                }
                _ => break,
            }
        }
        
        Ok(ComputeDecl {
            name,
            shader,
            threads_x: threads.0,
            threads_y: threads.1,
            threads_z: threads.2,
        })
    }
    
    fn parse_frame(&mut self) -> Result<FrameDecl, LangError> {
        let name = self.expect_identifier()?;
        
        if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            self.advance();
        }
        
        self.skip_newlines();
        
        let commands = self.parse_commands()?;
        
        Ok(FrameDecl { name, commands })
    }
    
    fn parse_queue(&mut self) -> Result<QueueDecl, LangError> {
        let name = self.expect_identifier()?;
        
        if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
            self.advance();
        }
        
        self.skip_newlines();
        
        let queue_type = match name.as_str() {
            "compute" => QueueType::Compute,
            "copy" => QueueType::Copy,
            _ => QueueType::Graphics,
        };
        
        let commands = self.parse_commands()?;
        
        Ok(QueueDecl { name, queue_type, commands })
    }
    
    fn parse_commands(&mut self) -> Result<Vec<Command>, LangError> {
        let mut commands = Vec::new();
        
        while let Some(token) = self.current() {
            match &token.kind {
                TokenKind::Clear => {
                    self.advance();
                    if matches!(self.peek_kind(), Some(TokenKind::Color)) {
                        self.advance();
                        let r = self.expect_float()? as f32;
                        let g = self.expect_float()? as f32;
                        let b = self.expect_float()? as f32;
                        let a = self.expect_float()? as f32;
                        commands.push(Command::ClearColor { r, g, b, a });
                    } else if matches!(self.peek_kind(), Some(TokenKind::Depth)) {
                        self.advance();
                        let depth = self.expect_float()? as f32;
                        commands.push(Command::ClearDepth { depth });
                    }
                }
                TokenKind::Viewport => {
                    self.advance();
                    let x = self.expect_integer()? as u32;
                    let y = self.expect_integer()? as u32;
                    let width = self.expect_integer()? as u32;
                    let height = self.expect_integer()? as u32;
                    commands.push(Command::Viewport { x, y, width, height });
                }
                TokenKind::Use => {
                    self.advance();
                    if matches!(self.peek_kind(), Some(TokenKind::Pipeline)) {
                        self.advance();
                        let name = self.expect_identifier()?;
                        commands.push(Command::UsePipeline { name });
                    } else if matches!(self.peek_kind(), Some(TokenKind::Compute)) {
                        self.advance();
                        let name = self.expect_identifier()?;
                        commands.push(Command::UseCompute { name });
                    }
                }
                TokenKind::Bind => {
                    self.advance();
                    let buffer = self.expect_identifier()?;
                    
                    let mut slot = 0u32;
                    let mut stride = 0u32;
                    
                    while let Some(t) = self.current() {
                        match &t.kind {
                            TokenKind::Slot => {
                                self.advance();
                                slot = self.expect_integer()? as u32;
                            }
                            TokenKind::Stride => {
                                self.advance();
                                stride = self.expect_integer()? as u32;
                            }
                            TokenKind::Newline => break,
                            _ => { self.advance(); break; }
                        }
                    }
                    
                    commands.push(Command::BindBuffer { buffer, slot, stride });
                }
                TokenKind::Draw => {
                    self.advance();
                    let count = self.expect_integer()? as u32;
                    commands.push(Command::Draw { vertex_count: count });
                }
                TokenKind::Dispatch => {
                    self.advance();
                    let x = self.expect_integer()? as u32;
                    let y = self.expect_integer()? as u32;
                    let z = self.expect_integer()? as u32;
                    commands.push(Command::Dispatch { x, y, z });
                }
                TokenKind::Present => {
                    self.advance();
                    commands.push(Command::Present);
                }
                TokenKind::Barrier => {
                    self.advance();
                    commands.push(Command::Barrier);
                }
                TokenKind::Wait => {
                    self.advance();
                    let queue = self.expect_identifier()?;
                    commands.push(Command::Wait { queue });
                }
                TokenKind::Signal => {
                    self.advance();
                    let queue = self.expect_identifier()?;
                    commands.push(Command::Signal { queue });
                }
                TokenKind::Newline => {
                    self.advance();
                }
                // End of command block
                TokenKind::Shader | TokenKind::Buffer | TokenKind::Texture |
                TokenKind::Pipeline | TokenKind::Compute | TokenKind::Frame | TokenKind::Queue => {
                    break;
                }
                _ => {
                    break;
                }
            }
        }
        
        Ok(commands)
    }
}
