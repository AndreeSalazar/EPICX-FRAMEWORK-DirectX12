# EPICX
## Autor: Eddi Andreé Salazar Matos

**React-inspired DirectX12 Graphics Framework for Rust**

EPICX es un framework de gráficos en Rust que encapsula DirectX12 con una arquitectura de componentes inspirada en React, facilitando el desarrollo de aplicaciones gráficas de alto rendimiento.

---

## ⚡ Tecnologías Revolucionarias (Migradas de ADead-GPU)

EPICX incluye tecnologías avanzadas migradas del proyecto **ADead-GPU**:

### 🎯 ADead-ISR (Intelligent Shading Rate)
**Adaptive Resolution Shading 2.0** - Ajusta automáticamente el detalle de píxeles (1x1 a 8x8) basado en importancia visual. **75% de ganancia de rendimiento**, mejor calidad que DLSS, **sin IA**, funciona en **CUALQUIER GPU**.

### ⚡ ADead-Vector3D (SDF Rendering)
**Renderizado 3D con Matemáticas Puras** - Inspirado en Adobe Illustrator. **Escalabilidad infinita**, **anti-aliasing perfecto**, **memoria mínima** (~1KB vs ~1MB para meshes).

### 🧮 ADead-AA (SDF Anti-Aliasing)
**Anti-Aliasing SDF** - Anti-aliasing matemático puro usando `fwidth()` y `smoothstep`. **Independiente de resolución**, **cero memoria extra**, **bordes perfectos**.

---

## 🏗️ Arquitectura Jerárquica

EPICX proporciona tres niveles de abstracción:

| Nivel | Módulo | Descripción |
|-------|--------|-------------|
| **A** | `dx12` | Wrappers crudos de DirectX12 - control total |
| **B** | `graphics` | Abstracciones intermedias - API más limpia |
| **C** | `easy` | API simplificada - uso muy general |

---

## Características

- **Arquitectura basada en componentes**: Construye UIs y gráficos usando componentes composables
- **Renderizado declarativo**: Describe qué quieres renderizar, no cómo hacerlo
- **Gestión de estado reactivo**: Las actualizaciones de estado disparan re-renders eficientes
- **Abstracción de DirectX12**: Todo el poder de DX12 sin la complejidad
- **Hooks estilo React**: `use_state`, `use_effect`, `use_memo`, `use_ref`
- **Sistema de temas**: Soporte para temas claros y oscuros
- **Lenguaje .gpu**: Parser para el lenguaje declarativo de ADead-GPU
- **SDF Primitives**: Esfera, Caja, Cilindro, Toro, Cápsula, Cono, Plano
- **Operaciones CSG**: Unión, Intersección, Sustracción (suaves)
- **Curvas Bézier 3D**: Cuadráticas y cúbicas en 3D
- **ISR**: Intelligent Shading Rate para rendimiento adaptativo

## Requisitos

- Windows 10/11
- Rust 1.70+
- GPU compatible con DirectX 12

## Instalación

Añade EPICX a tu `Cargo.toml`:

```toml
[dependencies]
epicx = { path = "." }
```

## Inicio Rápido

```rust
use epicx::prelude::*;

fn main() {
    // Crear la aplicación
    let app = App::builder()
        .title("Mi App EPICX")
        .size(1280, 720)
        .build();

    // Ejecutar con el componente raíz
    app.run(|| MyApp::new(())).unwrap();
}

// Definir un componente
struct MyApp {
    counter: i32,
}

impl Component for MyApp {
    type Props = ();
    type State = i32;

    fn new(_props: Self::Props) -> Self {
        Self { counter: 0 }
    }

    fn props(&self) -> &Self::Props { &() }
    fn state(&self) -> &Self::State { &self.counter }
    fn state_mut(&mut self) -> &mut Self::State { &mut self.counter }

    fn set_state<F>(&mut self, updater: F)
    where
        F: FnOnce(&mut Self::State),
    {
        updater(&mut self.counter);
    }

    fn render(&self, ctx: &mut RenderContext) -> Element {
        Element::group(vec![
            // Fondo
            Element::rect(ctx.viewport).fill(Color::from_hex(0x1a1a2e)),
            
            // Texto del contador
            Element::text(
                format!("Contador: {}", self.counter),
                ctx.width() / 2.0 - 50.0,
                ctx.height() / 2.0,
            ),
        ])
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
```

## Estructura del Proyecto

```
epicx/
├── src/
│   ├── lib.rs           # Punto de entrada de la biblioteca
│   ├── core/            # Sistema de componentes y elementos
│   │   ├── app.rs       # Aplicación principal
│   │   ├── component.rs # Trait Component
│   │   ├── element.rs   # Sistema de elementos (Virtual DOM)
│   │   ├── context.rs   # Contexto de renderizado
│   │   ├── state.rs     # Gestión de estado reactivo
│   │   └── props.rs     # Sistema de props
│   ├── dx12/            # Nivel A: Abstracción de DirectX12
│   │   ├── device.rs    # Dispositivo D3D12
│   │   ├── command_queue.rs
│   │   ├── swap_chain.rs
│   │   ├── pipeline.rs
│   │   ├── buffer.rs
│   │   ├── texture.rs
│   │   └── shader.rs
│   ├── graphics/        # Nivel B: Abstracciones intermedias
│   │   ├── mod.rs       # Graphics principal
│   │   ├── context.rs   # GraphicsContext
│   │   ├── frame.rs     # Frame management
│   │   └── resources.rs # GpuBuffer, GpuTexture, GpuMesh
│   ├── easy/            # Nivel C: API simplificada
│   │   └── mod.rs       # EasyApp, DrawContext, Sprite
│   ├── lang/            # Parser del lenguaje .gpu
│   │   ├── mod.rs       # API pública
│   │   ├── lexer.rs     # Tokenizador
│   │   ├── parser.rs    # Parser
│   │   └── ast.rs       # Árbol de sintaxis abstracta
│   ├── sdf/             # ADead-Vector3D: SDF Rendering
│   │   ├── mod.rs       # Ray marching, SdfScene
│   │   ├── primitives.rs # Sphere, Box3D, Cylinder, etc.
│   │   ├── operations.rs # Union, Intersection, Smooth ops
│   │   ├── bezier.rs    # Curvas y superficies Bézier
│   │   └── antialiasing.rs # SDF Anti-Aliasing
│   ├── isr/             # ADead-ISR: Intelligent Shading Rate
│   │   └── mod.rs       # IsrAnalyzer, ShadingRate
│   ├── components/      # Componentes predefinidos
│   ├── renderer/        # Sistema de renderizado
│   ├── window/          # Gestión de ventanas
│   ├── events/          # Sistema de eventos
│   ├── hooks/           # Hooks estilo React
│   └── math/            # Utilidades matemáticas
├── shaders/             # Shaders HLSL (de ADead-GPU)
│   ├── sdf_antialiasing.hlsl
│   ├── vector3d_raymarching.hlsl
│   └── cube_sdf.hlsl
├── examples/            # Ejemplos de uso
└── Cargo.toml
```

## Conceptos Clave

### Componentes

Los componentes son los bloques de construcción de EPICX. Similar a React, encapsulan estado, props y lógica de renderizado:

```rust
pub trait Component: Send + Sync + 'static {
    type Props: Props;
    type State: State;

    fn new(props: Self::Props) -> Self;
    fn render(&self, ctx: &mut RenderContext) -> Element;
    // ...
}
```

### Elementos

Los elementos representan el árbol de renderizado (similar al Virtual DOM de React):

```rust
// Crear elementos
let rect = Element::rect(Rect::new(0.0, 0.0, 100.0, 100.0))
    .fill(Color::RED)
    .stroke(Color::WHITE, 2.0);

let text = Element::text("Hola EPICX!", 10.0, 10.0);

let group = Element::group(vec![rect, text]);
```

### Hooks

EPICX proporciona hooks familiares para gestionar estado y efectos:

```rust
// Estado
let counter = use_state(0);
counter.set(counter.get() + 1);

// Efectos
use_effect(|| {
    println!("Componente montado!");
}, None);

// Memo
let expensive = use_memo(|| compute_expensive_value(), Some(vec![dep_hash]));

// Refs
let my_ref = use_ref(SomeValue::default());
```

### Contexto

Comparte datos a través del árbol de componentes:

```rust
// Proveer un tema
app.provide(Theme::dark());

// Consumir en un componente
fn render(&self, ctx: &mut RenderContext) -> Element {
    if let Some(theme) = ctx.use_context::<Theme>() {
        // Usar el tema
    }
}
```

## Componentes Predefinidos

- **Button**: Botón interactivo con estados hover/pressed
- **Container**: Contenedor con layout flex
- **Text**: Componente de texto
- **Image**: Componente de imagen
- **Canvas**: Lienzo para dibujo personalizado

## DirectX12

EPICX encapsula DirectX12 proporcionando abstracciones seguras:

```rust
// El Device maneja la GPU
let device = Device::new(true)?; // true = modo debug

// Command Queue para enviar comandos
let queue = CommandQueue::graphics(&device)?;

// Buffers para datos de vértices
let vertex_buffer = VertexBuffer::new(&device, size, stride)?;
vertex_buffer.write(&vertices)?;
```

## 🎮 API Easy (Nivel C) - Uso Simplificado

```rust
use epicx::easy::*;
use epicx::math::Color;

fn main() {
    let mut app = EasyApp::new("Mi Juego", 800, 600);
    
    app.run(|ctx| {
        ctx.clear(Color::from_hex(0x1a1a2e));
        ctx.fill_rect(100.0, 100.0, 200.0, 150.0, Color::RED);
        ctx.fill_circle(400.0, 300.0, 50.0, Color::GREEN);
        ctx.draw_text("¡Hola EPICX!", 50.0, 50.0);
    });
}
```

## 🎯 SDF Rendering (ADead-Vector3D)

```rust
use epicx::sdf::*;
use epicx::math::Vec3;

// Crear primitivas SDF
let sphere = Sphere::new(Vec3::ZERO, 1.0);
let cube = Box3D::cube(Vec3::new(2.0, 0.0, 0.0), 1.0);

// Operaciones CSG
let union = SmoothUnion::new(sphere, cube, 0.5);

// Ray marching
let config = RayMarchConfig::default();
let hit = ray_march(&union, camera_pos, ray_dir, &config);

if hit.hit {
    let color = calculate_lighting(hit.position, hit.normal);
}
```

## 📊 ISR (Intelligent Shading Rate)

```rust
use epicx::isr::*;

// Crear analizador ISR
let config = IsrConfig::default();
let mut analyzer = IsrAnalyzer::new(1920, 1080, config);

// Obtener shading rate para un tile
let rate = analyzer.get_tile_shading_rate(tile_x, tile_y);

match rate {
    ShadingRate::Full => { /* 1x1 - máxima calidad */ }
    ShadingRate::Half => { /* 2x2 - 75% menos trabajo */ }
    ShadingRate::Quarter => { /* 4x4 - 93% menos trabajo */ }
    ShadingRate::Eighth => { /* 8x8 - 98% menos trabajo */ }
}

// Estadísticas
let stats = analyzer.stats();
println!("Ahorro de rays: {}%", stats.savings_percent);
```

## 📝 Lenguaje .gpu (ADead-GPU)

```rust
use epicx::lang::*;

let source = r#"
shader vs "shaders/vertex.cso"
shader ps "shaders/pixel.cso"

buffer vertices f32x3 100 upload

pipeline render:
    vertex vs
    pixel ps
    topology triangles
    cull back
    depth on

frame main:
    clear color 0.1 0.1 0.15 1.0
    viewport 0 0 1280 720
    use pipeline render
    bind vertices slot 0 stride 12
    draw 100
    present
"#;

let program = parse_gpu_source(source)?;
println!("Shaders: {}", program.stats().shader_count);
println!("Comandos: {}", program.stats().total_commands);
```

## 🔬 Origen: ADead-GPU

Este proyecto incluye tecnologías migradas de **ADead-GPU**, un framework de investigación para GPU con:

- **83 tests pasando** (Compiler, Integration, Optimizer, Memory, Hot Reload, Multi-Queue, Profiler)
- **33% reducción de comandos** (optimizador)
- **71% ahorro de memoria** (aliasing + pooling)
- **75% ahorro de shading** (ISR adaptativo)
- **0.13ms hot reload** (actualizaciones en vivo)

Ver `ADead-GPU/README.md` para documentación completa del proyecto original.

## Licencia

MIT

## Contribuir

¡Las contribuciones son bienvenidas! Por favor, abre un issue o pull request.

---

**Built for understanding GPUs, pushing boundaries, and proving that mathematics beats brute force.**
