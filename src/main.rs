use anyhow::{Context, Result};
use klystron::{
    runtime_3d::{launch, App},
    DrawType, Engine, FramePacket, Material, Object, Vertex,
};
use notify::{watcher, DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use shaderc::Compiler;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use structopt::StructOpt;
use nalgebra::{Vector3, Matrix4};
mod motion;
use motion::PlayerMovement;

#[derive(Debug, StructOpt)]
#[structopt(name = "SDF Surfer", about = "Signed Distance Functions BUT SURFING BABEY")]
struct Opt {
    /// Use OpenXR backend
    #[structopt(short, long)]
    vr: bool,

    /// Use Wii balance board
    #[structopt(short, long)]
    balance: bool,

    /// Set shader directory (will look for glsl files to update, and will use those as fragment
    /// shaders)
    #[structopt(short, long)]
    shader_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Opt::from_args();
    launch::<MyApp>(args.vr, args)
}

struct MyApp {
    movement: PlayerMovement,
    fullscreen: Object,
    time: f32,
    compiler: Compiler,
    file_watch_rx: Receiver<DebouncedEvent>,
    _file_watcher: RecommendedWatcher,
}

impl App for MyApp {
    const NAME: &'static str = "Signed Distance Functions in 3D";

    type Args = Opt;

    fn new(engine: &mut dyn Engine, args: Self::Args) -> Result<Self> {
        // Set up file watch
        let (tx, file_watch_rx) = channel();
        let mut file_watcher = watcher(tx, Duration::from_millis(250))?;
        let parent_dir = args
            .shader_path
            .parent()
            .context("Shader has no parent dir?")?;
        file_watcher.watch(parent_dir, RecursiveMode::NonRecursive)?;

        // Create fullscreen mesh
        let (vertices, indices) = fullscreen_quad();
        let mesh = engine.add_mesh(&vertices, &indices)?;

        // Load initial material
        let mut compiler = Compiler::new().context("Failed to set up GLSL compiler")?;
        let material = load_shader(&args.shader_path, engine, &mut compiler)?;

        // Fullscreen quad
        let fullscreen = Object {
            mesh,
            material,
            transform: Matrix4::identity(),
        };

        Ok(Self {
            movement: PlayerMovement::new(args.balance)?,
            file_watch_rx,
            _file_watcher: file_watcher,
            compiler,
            fullscreen,
            time: 0.0,
        })
    }

    fn next_frame(&mut self, engine: &mut dyn Engine) -> Result<FramePacket> {
        // Reload shader on file change
        match self.file_watch_rx.try_recv() {
            Ok(DebouncedEvent::Create(p)) | Ok(DebouncedEvent::Write(p)) => {
                if p.is_file() && p.extension().map(|e| e == "frag").unwrap_or(false) {
                    match load_shader(&p, engine, &mut self.compiler) {
                        Ok(material) => {
                            let old = std::mem::replace(&mut self.fullscreen.material, material);
                            engine.remove_material(old)?;
                            println!("Loaded {:?}", p);
                        }
                        Err(e) => {
                            println!("ERROR: {}", e);
                        }
                    }
                }
            }
            _ => (),
        };

        engine.update_time_value(self.time)?;
        self.time += 0.01;

        Ok(FramePacket {
            objects: vec![self.fullscreen],
            base_transform: self.movement.player_transform(),
        })
    }
}

// Simple fullscreen vertex shader
const FULLSCREEN_VERT: &[u8] = include_bytes!("fullscreen.vert.spv");

fn load_shader(
    path: &PathBuf,
    engine: &mut dyn Engine,
    compiler: &mut Compiler,
) -> Result<Material> {
    let text = fs::read_to_string(path)?;
    let spirv = compiler.compile_into_spirv(
        &text,
        shaderc::ShaderKind::Fragment,
        path.to_str().unwrap(),
        "main",
        None,
    )?;
    engine.add_material(FULLSCREEN_VERT, spirv.as_binary_u8(), DrawType::Triangles)
}

fn fullscreen_quad() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        Vertex::new([-1.0, -1.0, 0.0], [1.; 3]),
        Vertex::new([-1.0, 1.0, 0.0], [1.; 3]),
        Vertex::new([1.0, -1.0, 0.0], [1.; 3]),
        Vertex::new([1.0, 1.0, 0.0], [1.; 3]),
    ];

    let indices = vec![2, 1, 0, 3, 1, 2];

    (vertices, indices)
}
