/*
 * File: src/window_manager.rs
 * Date: 06.11.2019
 * Author: MarkAtk
 *
 * MIT License
 *
 * Copyright (c) 2019 MarkAtk
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is furnished to do
 * so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use std::thread;
use std::time::Duration;
use std::sync::mpsc::{self, Sender, Receiver};
use tui::Terminal;
use tui::backend::Backend;
use crate::{Event, Window, EventResult};

pub struct WindowManager<T, I> where T: Backend, I: Send + 'static {
    pub tick_rate: u64,

    terminal: Terminal<T>,
    tx: Sender<Event<I>>,
    rx: Receiver<Event<I>>,
    windows: Vec<Box<dyn Window<T, I>>>
}

impl<T, I> WindowManager<T, I> where T: Backend, I: Send + 'static {
    pub fn new(terminal: Terminal<T>) -> WindowManager<T, I> {
        let (tx, rx) = mpsc::channel();

        WindowManager {
            tick_rate: 250,
            terminal,
            tx,
            rx,
            windows: vec!()
        }
    }

    pub fn get_tx(&self) -> &Sender<Event<I>> {
        &self.tx
    }

    pub fn push_window(&mut self, window: Box<dyn Window<T, I>>) {
        self.windows.push(window);
    }

    pub fn run(&mut self, initial_window: Box<dyn Window<T, I>>) -> Result<(), std::io::Error> {
        self.push_window(initial_window);

        // TODO: start input thread

        // start tick threads
        {
            let tx = self.tx.clone();
            let tick_rate = self.tick_rate;

            thread::spawn(move || {
                loop {
                    tx.send(Event::Tick).unwrap();

                    thread::sleep(Duration::from_millis(tick_rate));
                }
            });
        }

        // main loop
        while self.windows.is_empty() == false {
            if let Some(window) = self.windows.last_mut() {
                if window.should_close() {
                    self.windows.pop();

                    continue;
                }

                window.render(&mut self.terminal)?;

                let result = match self.rx.recv() {
                    // TODO: Add optional escape key handling?
                    Ok(Event::Input(event)) => Some(window.handle_key_event(event)),
                    Ok(Event::Tick) => Some(window.handle_tick(self.tick_rate)),
                    Ok(event) => Some(window.handle_event(event)),
                    _ => None
                };

                if let Some(event_result) = result {
                    self.apply_event_result(event_result);
                }
            }
        }

        Ok(())
    }

    fn apply_event_result(&mut self, event_result: EventResult<T, I>) {
        if event_result.remove {
            self.windows.pop();
        }

        if let Some(child) = event_result.child {
            self.windows.push(child);
        }
    }
}
