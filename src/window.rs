use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, self};
use std::sync::{mpsc, Arc};

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::Window;

use softbuffer::{Context, Surface};

pub struct WindowApp {
    window: Option<Rc<Window>>,
    context: Option<Rc<Context<Rc<Window>>>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    size: PhysicalSize<u32>,
    key_vector: [bool; 2],
    key_channel: mpsc::SyncSender<[bool; 2]>,
    exit: Arc<AtomicBool>,
}

impl ApplicationHandler for WindowApp {
    fn resumed(&mut self, ev_loop: &ActiveEventLoop) {
        println!("creating window");
        let win = Rc::new(
            ev_loop
                .create_window(Window::default_attributes())
                .expect("could not create window!"),
        );
        let ctx = Rc::new(Context::new(win.clone()).expect("could not create render context"));
        let sfc = Surface::new(&ctx.clone(), win.clone()).expect("could not create render surface");
        self.window = Some(win);
        self.context = Some(ctx);
        self.surface = Some(sfc);
        println!("window created");
    }

    fn window_event(
        &mut self,
        ev_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        println!("event: {event:?}");
        match event {
            WindowEvent::CloseRequested => {
                ev_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let mut buffer = self
                    .surface
                    .as_mut()
                    .unwrap()
                    .buffer_mut()
                    .expect("could not get surface buffer");
                // TODO if there's anything I actually want to put in the buffer
                buffer[0] = 0;
                buffer.present().expect("could not present display buffer");
            }
            WindowEvent::Resized(phy_size) => {
                self.size = phy_size;
                if self.size.width != 0 && self.size.height != 0 {
                    self.surface
                        .as_mut()
                        .unwrap()
                        .resize(
                            NonZeroU32::new(self.size.width).unwrap(),
                            NonZeroU32::new(self.size.height).unwrap(),
                        )
                        .expect("could not resize window");
                }
            }
            WindowEvent::KeyboardInput { event, .. } => match event {
                KeyEvent {
                    logical_key: Key::Named(key @ (NamedKey::ArrowLeft | NamedKey::ArrowRight)),
                    state,
                    ..
                } => {
                    let idx = if key == NamedKey::ArrowLeft { 0 } else { 1 };
                    match state {
                        ElementState::Pressed => {
                            self.key_vector[idx] = true;
                        }
                        ElementState::Released => {
                            self.key_vector[idx] = false;
                        }
                    }
                    self.key_channel.send(self.key_vector).expect("could not send data to other thread");
                }
                _ => (),
            },
            _ => {}
        }
        if self.exit.load(atomic::Ordering::Relaxed) {
            ev_loop.exit();
        }

    }
}

pub fn init_window(key_channel: mpsc::SyncSender<[bool; 2]>, exit: Arc<AtomicBool>) -> (EventLoop<()>, WindowApp) {
    let ev_loop = EventLoop::new().expect("could not create windowing event loop!");
    ev_loop.set_control_flow(ControlFlow::Poll);
    let app = WindowApp {
        window: None,
        context: None,
        surface: None,
        size: Default::default(),
        key_vector: [false; 2],
        key_channel,
        exit,
    };
    (ev_loop, app)
}
