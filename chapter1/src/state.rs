use winit::window::Window;

pub struct State<'window> {
  pub surface: wgpu::Surface<'window>,
  pub device: wgpu::Device,
  pub queue: wgpu::Queue,
  pub config: wgpu::SurfaceConfiguration,
  pub render_pipeline: wgpu::RenderPipeline,
  pub size: winit::dpi::PhysicalSize<u32>,
}

impl<'window> State<'window> {
  pub async fn new(window: &'window Window) -> Self {
    let size = window.inner_size();

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

    let (device, queue) = adapter
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
      .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);

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

    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("Shader"),
      source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/triangle.wgsl").into()),
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: None,
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        buffers: &[],
        compilation_options: wgpu::PipelineCompilationOptions {
          ..Default::default()
        },
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: config.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: wgpu::PipelineCompilationOptions {
          ..Default::default()
        },
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        ..Default::default()
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      multiview: None,
      cache: None,
    });

    Self {
      surface,
      device,
      queue,
      config,
      render_pipeline,
      size,
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

  pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
    let output = self.surface.get_current_texture()?;
    let view = output
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    // GPU에 전달할 그래픽스 명령들을 기록하기 위한 인코더 생성
    // Encoder for recording commands for GPU.
    let mut encoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
      });

    {
      // 렌더링 명령을 기록하기 위한 렌더 패스 생성
      // Render pass for recording rendering commands.
      let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            // 렌더링을 시작할 때 출력 첨부물에 대해 수행할 작업입니다.
            // Operation to perform to the output attachment at the start of a render pass.
            load: wgpu::LoadOp::Clear(wgpu::Color {
              r: 0.0,
              g: 0.0,
              b: 0.0,
              a: 1.0,
            }),
            // 렌더링 결과를 출력 첨부물에 저장합니다.
            // Operation to perform to the output attachment at the end of a render pass.
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
      });

      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.draw(0..3, 0..1);
    }

    // 커맨드 버퍼를 GPU 큐에 제출
    // Submit the command buffer to the GPU queue.
    self.queue.submit(std::iter::once(encoder.finish()));

    output.present();

    Ok(())
  }
}
