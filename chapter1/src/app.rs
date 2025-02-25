use crate::state::State;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

#[derive(Default)]
pub struct App {
  window: Option<Window>,
  state: Option<State<'static>>,
}

impl ApplicationHandler for App {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    self.window = Some(
      event_loop
        .create_window(Window::default_attributes())
        .unwrap(),
    );

    // 렌더링을 위한 상태 생성
    // 참고: 이것은 라이프타임 핵을 사용합니다. 하지만 실제 프로덕션 코드에서는
    // Rc/Arc를 사용하거나 코드 구조를 재구성해야 할 수 있습니다.
    let window_ref = self.window.as_ref().unwrap();
    let window_ptr: *const Window = window_ref;
    let window_ref_unsafe: &'static Window = unsafe { &*window_ptr };

    // 비동기적으로 상태 초기화
    pollster::block_on(async {
      self.state = Some(State::new(window_ref_unsafe).await);
    });
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::RedrawRequested => {
        if let Some(state) = &mut self.state {
          match state.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
            Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
            Err(e) => eprintln!("{:?}", e),
          }
        }

        self.window.as_ref().unwrap().request_redraw();
      }
      WindowEvent::Resized(physical_size) => {
        if let Some(state) = &mut self.state {
          state.resize(physical_size);
        }
      }
      _ => (),
    }
  }
}
