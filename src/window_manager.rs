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
use std::io::{self, Write};
use std::time::Duration;
use std::sync::mpsc::{self, Sender, Receiver};
use tui::Terminal;
use tui::backend::{self, Backend};
use crate::{Event, Window, EventResult};

#[cfg(feature = "termion-backend")]
use termion::event::Key;

#[cfg(feature = "crossterm-backend")]
use crossterm::event::KeyEvent;
#[cfg(feature = "crossterm-backend")]
use crossterm::{terminal, execute};

#[derive(Debug)]
pub struct WindowManagerSettings {
    pub show_cursor: bool,
    pub raw_mode: bool,
    pub alternate_screen: bool
}

impl Default for WindowManagerSettings {
    fn default() -> Self {
        WindowManagerSettings {
            show_cursor: true,
            raw_mode: false,
            alternate_screen: false
        }
    }
}

pub struct WindowManager<T, I> where T: Backend + Write, I: Send + 'static {
    pub tick_rate: u64,

    terminal: Terminal<T>,
    tx: Sender<Event<I>>,
    rx: Receiver<Event<I>>,
    windows: Vec<Box<dyn Window<T, I>>>,
    settings: WindowManagerSettings
}

impl<T, I> WindowManager<T, I> where T: Backend + Write, I: Send + 'static {
    pub fn new_with_backend(backend: T, settings: WindowManagerSettings) -> Result<Self, io::Error> {
        let (tx, rx) = mpsc::channel();
        let mut terminal = Terminal::new(backend)?;

        if settings.show_cursor == false {
            terminal.hide_cursor()?;
        }

        Ok(WindowManager {
            tick_rate: 250,
            terminal,
            tx,
            rx,
            windows: vec!(),
            settings
        })
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
                    //Ok(event) => Some(window.handle_event(event)),
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

    #[cfg(feature = "termion-backend")]
    fn cleanup(&mut self) {

    }

    #[cfg(feature = "crossterm-backend")]
    fn cleanup(&mut self) {
        if self.settings.raw_mode {
            terminal::disable_raw_mode().unwrap();
        }

        if self.settings.alternate_screen {
            execute!(io::stdout(), terminal::LeaveAlternateScreen).unwrap();
        }
    }
}

impl<T, I> Drop for WindowManager<T, I> where T: Backend + Write, I: Send + 'static {
    fn drop(&mut self) {
        self.cleanup();

        if self.settings.show_cursor == false {
            self.terminal.show_cursor().unwrap();
        }
    }
}

#[cfg(feature = "termion-backend")]
impl WindowManager<backend::TermionBackend<io::Stdout>, Key> {
    pub fn new(settings: WindowManagerSettings) -> Result<Self, io::Error> {
        let stdout = io::stdout();
        let backend = backend::TermionBackend::new(stdout);

        WindowManager::new_with_backend(backend, settings)
    }
}

#[cfg(feature = "crossterm-backend")]
impl WindowManager<backend::CrosstermBackend<io::Stdout>, KeyEvent> {
    pub fn new(settings: WindowManagerSettings) -> Result<Self, io::Error> {
        if settings.raw_mode {
            if let Err(err) = terminal::enable_raw_mode() {
                return Err(io::Error::new(io::ErrorKind::Other, err));
            }
        }

        let mut stdout = io::stdout();

        if settings.alternate_screen {
            if let Err(err) = execute!(stdout, terminal::EnterAlternateScreen) {
                return Err(io::Error::new(io::ErrorKind::Other, err));
            }
        }

        let backend = backend::CrosstermBackend::new(stdout);

        WindowManager::new_with_backend(backend, settings)
    }
}
