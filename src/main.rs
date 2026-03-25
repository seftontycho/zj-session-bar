use std::collections::BTreeMap;
use std::path::PathBuf;
use zellij_tile::prelude::*;
use zellij_tile_utils::style;

#[derive(Default)]
struct SessionBar {
    sessions: Vec<SessionInfo>,
    current_session: String,
    mode_info: ModeInfo,
}

register_plugin!(SessionBar);

static ARROW_SEPARATOR: &str = "\u{E0B0}";

impl ZellijPlugin for SessionBar {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        // Permissions are granted via _allow_exec_host_cmd in the layout.
        subscribe(&[EventType::SessionUpdate, EventType::ModeUpdate]);
        set_selectable(false);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::SessionUpdate(sessions, _) => {
                if let Some(current) = sessions.iter().find(|s| s.is_current_session) {
                    self.current_session = current.name.clone();
                }
                self.sessions = sessions;
                true
            }
            Event::ModeUpdate(mode_info) => {
                self.mode_info = mode_info;
                true
            }
            _ => false,
        }
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        match pipe_message.name.as_str() {
            "switch_session" => {
                if let Some(name) = pipe_message.payload {
                    let layout = pipe_message.args.get("layout").cloned();
                    let cwd = pipe_message.args.get("cwd").map(PathBuf::from);
                    if let Some(layout_path) = layout {
                        switch_session_with_layout(
                            Some(&name),
                            LayoutInfo::File(layout_path),
                            cwd,
                        );
                    } else {
                        switch_session(Some(&name));
                    }
                }
                false
            }
            "next_session" => {
                self.switch_relative(1);
                false
            }
            "prev_session" => {
                self.switch_relative(-1);
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, _cols: usize) {
        if self.sessions.is_empty() {
            return;
        }

        let palette = self.mode_info.style.colors;
        let separator = ARROW_SEPARATOR;

        let sep_color = palette.text_unselected.background;
        let text_color = palette.text_unselected.base;
        let mut output = String::new();

        // Prefix label, same style as the tab-bar's "Zellij (session)" prefix.
        let prefix = style!(text_color, sep_color).bold().paint(" Sessions ");
        output.push_str(&prefix.to_string());

        for session in &self.sessions {
            let is_active = session.name == self.current_session;

            let bg_color = if is_active {
                palette.ribbon_selected.background
            } else {
                palette.ribbon_unselected.background
            };
            let fg_color = if is_active {
                palette.ribbon_selected.base
            } else {
                palette.ribbon_unselected.base
            };

            let left_sep = style!(sep_color, bg_color).paint(separator);
            let text = style!(fg_color, bg_color)
                .bold()
                .paint(format!(" {} ", session.name));
            let right_sep = style!(bg_color, sep_color).paint(separator);

            output.push_str(&left_sep.to_string());
            output.push_str(&text.to_string());
            output.push_str(&right_sep.to_string());
        }

        // Fill remaining space with bar background.
        match sep_color {
            PaletteColor::Rgb((r, g, b)) => {
                print!("{}\u{1b}[48;2;{};{};{}m\u{1b}[0K", output, r, g, b);
            }
            PaletteColor::EightBit(color) => {
                print!("{}\u{1b}[48;5;{}m\u{1b}[0K", output, color);
            }
        }
    }
}

impl SessionBar {
    fn switch_relative(&self, offset: isize) {
        if self.sessions.is_empty() {
            return;
        }
        let current_idx = self
            .sessions
            .iter()
            .position(|s| s.name == self.current_session)
            .unwrap_or(0);
        let len = self.sessions.len() as isize;
        let next_idx = ((current_idx as isize + offset) % len + len) % len;
        let next = &self.sessions[next_idx as usize];
        switch_session(Some(&next.name));
    }
}
