use std::cell::RefCell;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window, dpi::PhysicalSize,
};
use std::sync::mpsc;
use std::thread;
use std::rc::Rc;

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
pub enum LoopGameEvent {
    Redraw,
    Resize(PhysicalSize<u32>),
    Exit,
    None
}

impl LoopGameBase {
    pub fn run(self, tx : mpsc::Sender<LoopGameEvent>)
    {
        let handle = self.event_loop.run(move |event, _, control_flow| {

            *control_flow = ControlFlow::Poll;

            self.window.request_redraw();

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
                Event::RedrawRequested(id) => {
                    tx.send(LoopGameEvent::Redraw).unwrap();
                }
                _ => {}
            }
        });


    }
}

pub trait LoopGame {
    
    fn init(&mut self, base : &LoopGameBase);
    fn logick_loop(&mut self);
    fn draw_loop(&mut self);
    fn resize_event(&mut self, size : &PhysicalSize<u32>);
}