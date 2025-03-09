mod rasterization;
mod state;

use state::State;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
pub struct GraphicsApplication {
  window: Option<Window>,
  state: Option<State<'static>>,
}

impl ApplicationHandler for GraphicsApplication {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    self.window = Some(
      event_loop
        .create_window(Window::default_attributes())
        .unwrap(),
    );

    // 렌더링을 위한 상태 생성
    // 참고: 이것은 라이프타임 핵을 사용합니다. 하지만 실제 프로덕션 코드에서는
    // Rc/Arc를 사용하거나 코드 구조를 재구성해야 할 수 있습니다.
    let window = self.window.as_ref().unwrap();
    let window: *const Window = window;
    let window: &'static Window = unsafe { &*window };

    // 비동기적으로 상태 초기화
    pollster::block_on(async {
      self.state = Some(State::new(window).await);
    });
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    if let Some(state) = &mut self.state {
      let window = self.window.as_ref().unwrap();

      let response = state.egui_state.on_window_event(window, &event);
      if response.consumed {
        return;
      }
    }

    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::RedrawRequested => {
        if let Some(state) = &mut self.state {
          let window = self.window.as_ref().unwrap();

          match state.render(window) {
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
      WindowEvent::KeyboardInput { event, .. } => {
        if event.state == ElementState::Pressed {
          println!("Key pressed: {:?}", event.physical_key);
        }
      }
      // WindowEvent::MouseInput { state, button, .. } => {
      //   if state == ElementState::Released {
      //     match button {
      //       MouseButton::Left => println!("Left mouse button released"),
      //       MouseButton::Right => println!("Right mouse button released"),
      //       _ => {}
      //     }
      //   }
      // }
      // WindowEvent::CursorMoved { position, .. } => {
      //   println!("Mouse moved to: ({}, {})", position.x, position.y);
      // }
      _ => (),
    }
  }
}

fn main() {
  let event_loop = EventLoop::new().unwrap();
  event_loop.set_control_flow(ControlFlow::Wait);
  let mut app = GraphicsApplication::default();
  let _ = event_loop.run_app(&mut app);
}
