use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window, dpi::PhysicalSize,
};
use std::sync::mpsc;

pub struct LoopGameBase {
    pub window : Window,
    pub event_loop : EventLoop<()>
}

impl Default for LoopGameBase {
    fn default() -> Self {
        let event_loop = EventLoop::new();
        let window = winit::window::Window::new(&event_loop).unwrap();

        Self {
            window,
            event_loop
        }
    }
}

#[derive(Clone)]
enum LoopGameEvent {
    Redraw,
    Resize(PhysicalSize<u32>),
    Exit,
    None
}

impl LoopGameBase {
    pub fn run<Game>(self, game : &mut Game) where Game : LoopGame
    {
        let (tx, rx) = mpsc::channel();
        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    tx.send(LoopGameEvent::Resize(size)).unwrap();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    tx.send(LoopGameEvent::Exit).unwrap();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });

        game.init(&self);

        let mut game_runnng = true;

        while game_runnng {
            game.logick_loop(&mut self);

            match rx.recv() {
                Ok(data) => {
                    match data {
                        LoopGameEvent::Redraw => {
                            game.draw_loop(&mut self);
                        }
                        LoopGameEvent::Resize(size) => {
                            game.resize_event(&mut self, &size);
                        }
                        LoopGameEvent::Exit => {
                            game_runnng = false;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}


pub trait LoopGame {
    
    fn init(&mut self, base : &LoopGameBase);
    fn logick_loop(&mut self, base : &mut LoopGameBase);
    fn draw_loop(&mut self, base : &mut LoopGameBase);
    fn resize_event(&mut self, base : &mut LoopGameBase, size : &PhysicalSize<u32>);
}