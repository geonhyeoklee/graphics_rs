use egui_wgpu::ScreenDescriptor;
use wgpu::util::DeviceExt;
use winit::window::Window;

// Vertex 데이터 구조체
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
  position: [f32; 4],
  uv: [f32; 2],
}

pub struct State<'window> {
  pub surface: wgpu::Surface<'window>,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub config: wgpu::SurfaceConfiguration,
  pub render_pipeline: wgpu::RenderPipeline,
  pub size: winit::dpi::PhysicalSize<u32>,
  egui_renderer: egui_wgpu::Renderer,
  pub egui_state: egui_winit::State,
  pub egui_ctx: egui::Context,
  start_time: std::time::Instant,
  vertex_buffer: wgpu::Buffer,
}

impl<'window> State<'window> {
  pub async fn new(window: &'window Window) -> Self {
    let size = window.inner_size();

    let (_instance, surface, adapter) = Self::initialize_wgpu(window).await;
    let (device, queue) = Self::create_device_queue(&adapter).await;
    let config = Self::configure_surface(&surface, &adapter, &device, size);
    let render_pipeline = Self::create_render_pipeline(&device, &config);
    let (egui_ctx, egui_state, egui_renderer) = Self::initialize_egui(window, &device, &config);
    let vertex_buffer = Self::create_vertex_buffer(&device);

    Self {
      surface,
      device,
      queue,
      config,
      render_pipeline,
      size,
      egui_renderer,
      egui_state,
      egui_ctx,
      start_time: std::time::Instant::now(),
      vertex_buffer,
    }
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.size = new_size;
      self.config.width = new_size.width;
      self.config.height = new_size.height;
      self.surface.configure(&self.device, &self.config);
    }
  }

  pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    let full_output = self.update_egui(window);
    let (clipped_meshes, screen_descriptor) = self.prepare_egui_meshes(window, full_output);
    let command_buffer = self.render_frame(&view, &clipped_meshes, &screen_descriptor);

    self.queue.submit(Some(command_buffer));
    output.present();

    Ok(())
  }

  async fn initialize_wgpu(
    window: &'window Window,
  ) -> (wgpu::Instance, wgpu::Surface<'window>, wgpu::Adapter) {
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(window).unwrap();
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
      })
      .await
      .unwrap();
    (instance, surface, adapter)
  }

  async fn create_device_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
      .request_device(
        &wgpu::DeviceDescriptor {
          required_features: wgpu::Features::empty(),
          required_limits: wgpu::Limits::default(),
          label: None,
          memory_hints: wgpu::MemoryHints::Performance,
        },
        None,
      )
      .await
      .unwrap()
  }

  fn configure_surface(
    surface: &wgpu::Surface, adapter: &wgpu::Adapter, device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
  ) -> wgpu::SurfaceConfiguration {
    let surface_caps = surface.get_capabilities(adapter);
    let surface_format = surface_caps.formats[0];

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
      alpha_mode: surface_caps.alpha_modes[0],
      view_formats: Vec::new(),
      desired_maximum_frame_latency: 2,
    };

    surface.configure(device, &config);
    config
  }

  fn create_render_pipeline(
    device: &wgpu::Device, config: &wgpu::SurfaceConfiguration,
  ) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/vertex_shader.wgsl").into()),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: None,
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        buffers: &[wgpu::VertexBufferLayout {
          array_stride: std::mem::size_of::<[f32; 6]>() as u64,
          step_mode: wgpu::VertexStepMode::Vertex,
          attributes: &[
            wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x4,
              offset: 0,
              shader_location: 0,
            },
            wgpu::VertexAttribute {
              format: wgpu::VertexFormat::Float32x2,
              offset: 16,
              shader_location: 1,
            },
          ],
        }],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: config.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        ..Default::default()
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: None,
    })
  }

  fn initialize_egui(
    window: &Window, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration,
  ) -> (egui::Context, egui_winit::State, egui_wgpu::Renderer) {
    let egui_ctx = egui::Context::default();
    let viewport_id = egui::ViewportId::ROOT;
    let egui_state =
      egui_winit::State::new(egui_ctx.clone(), viewport_id, window, None, None, None);
    let egui_renderer = egui_wgpu::Renderer::new(device, config.format, None, 1, false);
    (egui_ctx, egui_state, egui_renderer)
  }

  fn create_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&[
        Vertex {
          position: [-0.5, -0.5, 0.0, 1.0],
          uv: [0.0, 0.0],
        },
        Vertex {
          position: [0.5, -0.5, 0.0, 1.0],
          uv: [1.0, 0.0],
        },
        Vertex {
          position: [0.0, 0.5, 0.0, 1.0],
          uv: [0.5, 1.0],
        },
      ]),
      usage: wgpu::BufferUsages::VERTEX,
    })
  }

  fn update_egui(&mut self, window: &Window) -> egui::FullOutput {
    self
      .egui_ctx
      .run(self.egui_state.take_egui_input(window), |ctx| {
        egui::Window::new("Controls").show(ctx, |ui| {
          ui.label("Hello from egui!");
          ui.label(format!(
            "Time: {:.1}s",
            self.start_time.elapsed().as_secs_f32()
          ));
        });
      })
  }

  fn prepare_egui_meshes(
    &mut self, window: &Window, full_output: egui::FullOutput,
  ) -> (Vec<egui::ClippedPrimitive>, ScreenDescriptor) {
    self
      .egui_state
      .handle_platform_output(window, full_output.platform_output);

    let screen_descriptor = ScreenDescriptor {
      size_in_pixels: [self.size.width, self.size.height],
      pixels_per_point: window.scale_factor() as f32,
    };

    let clipped_meshes = self
      .egui_ctx
      .tessellate(full_output.shapes, window.scale_factor() as f32);
    (clipped_meshes, screen_descriptor)
  }

  fn render_frame(
    &mut self, view: &wgpu::TextureView, meshes: &[egui::ClippedPrimitive],
    screen_descriptor: &ScreenDescriptor,
  ) -> wgpu::CommandBuffer {
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    self.egui_renderer.update_buffers(
      &self.device,
      &self.queue,
      &mut encoder,
      meshes,
      screen_descriptor,
    );

    {
      let desc = wgpu::RenderPassDescriptor {
        label: Some("Egui main render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      };

      let render_pass = encoder.begin_render_pass(&desc);
      let render_pass = &mut render_pass.forget_lifetime();

      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw(0..3, 0..1);

      self
        .egui_renderer
        .render(render_pass, meshes, screen_descriptor);
    }

    encoder.finish()
  }
}
