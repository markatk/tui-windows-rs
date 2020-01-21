/*
 * File: examples/crossterm.rs
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

use std::io::{Stdout, Error};
use tui_windows::{WindowManager, WindowManagerSettings, Window, EventResult};
use tui::Terminal;
use tui::widgets::{Widget, Paragraph, Block, Borders, Text};
use tui::layout::{Layout, Direction, Constraint};
use tui::backend::CrosstermBackend;
use crossterm::event::{KeyEvent, KeyCode};

struct MainWindow {
    should_close: bool
}

impl MainWindow {
    pub fn new() -> Box<Self> {
        Box::new(MainWindow {
            should_close: false
        })
    }
}

impl Window<CrosstermBackend<Stdout>, KeyEvent> for MainWindow {
    fn render(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Error> {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0)
                ].as_ref())
                .split(f.size());

            Paragraph::new([Text::raw("Example crossterm window")].iter())
                .block(Block::default()
                    .title("tui-windows")
                    .borders(Borders::ALL))
                .render(&mut f, chunks[0]);
        })
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> EventResult<CrosstermBackend<Stdout>, KeyEvent> {
        match event {
            KeyEvent { code: KeyCode::Esc, modifiers: _ } => self.should_close = true,
            _ => ()
        }

        EventResult::new()
    }

    fn should_close(&self) -> bool {
        self.should_close
    }
}

fn main() {
    let mut window_manager = WindowManager::new(WindowManagerSettings {
        show_cursor: true,
        raw_mode: true,
        alternate_screen: true
    }).unwrap();

    let window = MainWindow::new();

    window_manager.run(window).unwrap();
}
