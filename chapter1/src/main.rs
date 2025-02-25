mod app;
mod state;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
  let event_loop = EventLoop::new().unwrap();

  // ControlFlow::Wait는 처리할 이벤트가 없으면 이벤트 루프를 일시 중지합니다.
  event_loop.set_control_flow(ControlFlow::Wait);

  let mut app = App::default();
  let _ = event_loop.run_app(&mut app);
}
