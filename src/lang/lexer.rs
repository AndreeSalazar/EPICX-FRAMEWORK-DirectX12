//! Lexer for .gpu language

use std::iter::Peekable;
use std::str::Chars;

/// Token kinds
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Shader,
    Buffer,
    Texture,
    Pipeline,
    Compute,
    Frame,
    Queue,
    
    // Pipeline keywords
    Vertex,
    Pixel,
    Geometry,
    Topology,
    Cull,
    Depth,
    Blend,
    Threads,
    
    // Commands
    Clear,
    Color,
    Viewport,
    Scissor,
    Use,
    Bind,
    Draw,
    Dispatch,
    Present,
    Barrier,
    Wait,
    Signal,
    Slot,
    Stride,
    
    // Types
    F32,
    F32x2,
    F32x3,
    F32x4,
    U32,
    I32,
    U16,
    Mat4,
    RGBA8,
    RGBA16F,
    RGBA32F,
    
    // Heap types
    Default,
    Upload,
    Readback,
    
    // Topology
    Triangles,
    TriangleStrip,
    Lines,
    Points,
    
    // Cull modes
    None,
    Front,
    Back,
    
    // Blend modes
    Alpha,
    Additive,
    Multiply,
    
    // Queue types
    Graphics,
    Copy,
    
    // Values
    On,
    Off,
    
    // Literals
    Identifier(String),
    String(String),
    Integer(i64),
    Float(f64),
    
    // Punctuation
    Colon,
    Newline,
    Indent,
    Dedent,
    
    // Special
    Comment,
    Eof,
}

/// A token with position information
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Self { kind, line, column }
    }
}

/// Lexer for .gpu source
pub struct Lexer<'a> {
    source: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    pending_dedents: usize,
    at_line_start: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.chars().peekable(),
            line: 1,
            column: 1,
            indent_stack: vec![0],
            pending_dedents: 0,
            at_line_start: true,
        }
    }
    
    fn peek(&mut self) -> Option<char> {
        self.source.peek().copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        let c = self.source.next();
        if let Some(ch) = c {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            } else {
                self.column += 1;
            }
        }
        c
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    fn read_string(&mut self) -> String {
        let mut s = String::new();
        self.advance(); // consume opening quote
        while let Some(c) = self.peek() {
            if c == '"' {
                self.advance();
                break;
            }
            s.push(self.advance().unwrap());
        }
        s
    }
    
    fn read_identifier(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        s
    }
    
    fn read_number(&mut self) -> TokenKind {
        let mut s = String::new();
        let mut is_float = false;
        
        // Handle negative
        if self.peek() == Some('-') {
            s.push(self.advance().unwrap());
        }
        
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(self.advance().unwrap());
            } else if c == '.' && !is_float {
                is_float = true;
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        
        if is_float {
            TokenKind::Float(s.parse().unwrap_or(0.0))
        } else {
            TokenKind::Integer(s.parse().unwrap_or(0))
        }
    }
    
    fn keyword_or_identifier(&self, s: &str) -> TokenKind {
        match s {
            // Keywords
            "shader" => TokenKind::Shader,
            "buffer" => TokenKind::Buffer,
            "texture" => TokenKind::Texture,
            "pipeline" => TokenKind::Pipeline,
            "compute" => TokenKind::Compute,
            "frame" => TokenKind::Frame,
            "queue" => TokenKind::Queue,
            
            // Pipeline
            "vertex" => TokenKind::Vertex,
            "pixel" => TokenKind::Pixel,
            "geometry" => TokenKind::Geometry,
            "topology" => TokenKind::Topology,
            "cull" => TokenKind::Cull,
            "depth" => TokenKind::Depth,
            "blend" => TokenKind::Blend,
            "threads" => TokenKind::Threads,
            
            // Commands
            "clear" => TokenKind::Clear,
            "color" => TokenKind::Color,
            "viewport" => TokenKind::Viewport,
            "scissor" => TokenKind::Scissor,
            "use" => TokenKind::Use,
            "bind" => TokenKind::Bind,
            "draw" => TokenKind::Draw,
            "dispatch" => TokenKind::Dispatch,
            "present" => TokenKind::Present,
            "barrier" => TokenKind::Barrier,
            "wait" => TokenKind::Wait,
            "signal" => TokenKind::Signal,
            "slot" => TokenKind::Slot,
            "stride" => TokenKind::Stride,
            
            // Types
            "f32" => TokenKind::F32,
            "f32x2" => TokenKind::F32x2,
            "f32x3" => TokenKind::F32x3,
            "f32x4" => TokenKind::F32x4,
            "u32" => TokenKind::U32,
            "i32" => TokenKind::I32,
            "u16" => TokenKind::U16,
            "mat4" => TokenKind::Mat4,
            "rgba8" => TokenKind::RGBA8,
            "rgba16f" => TokenKind::RGBA16F,
            "rgba32f" => TokenKind::RGBA32F,
            
            // Heap types
            "default" => TokenKind::Default,
            "upload" => TokenKind::Upload,
            "readback" => TokenKind::Readback,
            
            // Topology
            "triangles" => TokenKind::Triangles,
            "trianglestrip" => TokenKind::TriangleStrip,
            "lines" => TokenKind::Lines,
            "points" => TokenKind::Points,
            
            // Cull
            "none" => TokenKind::None,
            "front" => TokenKind::Front,
            "back" => TokenKind::Back,
            
            // Blend
            "alpha" => TokenKind::Alpha,
            "additive" => TokenKind::Additive,
            "multiply" => TokenKind::Multiply,
            
            // Queue types
            "graphics" => TokenKind::Graphics,
            "copy" => TokenKind::Copy,
            
            // Values
            "on" => TokenKind::On,
            "off" => TokenKind::Off,
            
            _ => TokenKind::Identifier(s.to_string()),
        }
    }
    
    fn next_token(&mut self) -> Option<Token> {
        // Handle pending dedents
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            return Some(Token::new(TokenKind::Dedent, self.line, self.column));
        }
        
        self.skip_whitespace();
        
        let line = self.line;
        let column = self.column;
        
        match self.peek()? {
            '#' => {
                // Comment - skip to end of line
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                self.next_token()
            }
            '\n' => {
                self.advance();
                Some(Token::new(TokenKind::Newline, line, column))
            }
            ':' => {
                self.advance();
                Some(Token::new(TokenKind::Colon, line, column))
            }
            '"' => {
                let s = self.read_string();
                Some(Token::new(TokenKind::String(s), line, column))
            }
            c if c.is_ascii_digit() || c == '-' => {
                let kind = self.read_number();
                Some(Token::new(kind, line, column))
            }
            c if c.is_alphabetic() || c == '_' => {
                let s = self.read_identifier();
                let kind = self.keyword_or_identifier(&s);
                Some(Token::new(kind, line, column))
            }
            _ => {
                self.advance();
                self.next_token()
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}
