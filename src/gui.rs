use std::path::PathBuf;

use iced::widget::{button, column, container, progress_bar, row, scrollable, text, text_input};
use iced::{Element, Length, Task};

use crate::fs_scan::{FsEntry, scan_path};
use crate::layout::human_size;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub enum Message {
    RootChanged(String),
    ScanPressed,
    RefreshPressed,
    EnterChild(usize),
    GoParent,
}

pub struct GuiApp {
    root_input: String,
    state: Option<AppState>,
    error: Option<String>,
}

impl Default for GuiApp {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let root_input = cwd.display().to_string();
        let mut app = Self {
            root_input,
            state: None,
            error: None,
        };
        app.scan_current_root();
        app
    }
}

impl GuiApp {
    fn scan_current_root(&mut self) {
        match init_state_from_path(self.root_input.trim()) {
            Ok(root) => {
                self.state = Some(root);
                self.error = None;
            }
            Err(err) => {
                self.error = Some(err);
            }
        }
    }
}

pub fn init_state_from_path(path: &str) -> Result<AppState, String> {
    let root = scan_path(&PathBuf::from(path)).map_err(|e| e.to_string())?;
    Ok(AppState::new(root))
}

pub fn update(app: &mut GuiApp, message: Message) -> Task<Message> {
    match message {
        Message::RootChanged(value) => app.root_input = value,
        Message::ScanPressed | Message::RefreshPressed => app.scan_current_root(),
        Message::EnterChild(idx) => {
            if let Some(state) = app.state.as_mut() {
                state.selected_index = idx;
                state.enter_selected();
            }
        }
        Message::GoParent => {
            if let Some(state) = app.state.as_mut() {
                state.go_parent();
            }
        }
    }
    Task::none()
}

pub fn view(app: &GuiApp) -> Element<'_, Message> {
    let controls = row![
        text_input("Path", &app.root_input)
            .on_input(Message::RootChanged)
            .on_submit(Message::ScanPressed)
            .padding(8)
            .width(Length::Fill),
        button("Scan").on_press(Message::ScanPressed),
        button("Refresh").on_press(Message::RefreshPressed),
        button("Up").on_press(Message::GoParent),
    ]
    .spacing(8);

    let body: Element<'_, Message> = if let Some(state) = &app.state {
        let children = state.current_children();
        let total = children.iter().map(|e| e.size).sum::<u64>().max(1);
        let mut list_col =
            column![text(format!("Current: {}", state.current_dir.display()))].spacing(6);
        for (idx, child) in children.iter().enumerate() {
            let kind = match child.kind {
                crate::fs_scan::EntryKind::Directory => "d",
                crate::fs_scan::EntryKind::File => "f",
            };
            list_col = list_col.push(
                button(text(format!(
                    "[{kind}] {}  {}",
                    child.name,
                    human_size(child.size)
                )))
                .on_press(Message::EnterChild(idx))
                .width(Length::Fill),
            );
        }
        let left = scrollable(list_col).height(Length::Fill);

        let mut viz_col = column![text("Usage Visualization")].spacing(8);
        for child in children.iter().take(40) {
            viz_col = viz_col.push(usage_row(child, total));
        }
        let right = scrollable(viz_col).height(Length::Fill);
        row![
            container(left).width(Length::FillPortion(1)),
            container(right).width(Length::FillPortion(2)),
        ]
        .spacing(10)
        .into()
    } else {
        container(text("No data")).into()
    };

    let mut root = column![controls, body].spacing(10).padding(10);
    if let Some(err) = &app.error {
        root = root.push(text(format!("Error: {err}")));
    }
    root.into()
}

fn usage_row(entry: &FsEntry, total: u64) -> Element<'_, Message> {
    let ratio = (entry.size as f32 / total as f32).clamp(0.0, 1.0);
    row![
        text(format!("{} ({})", entry.name, human_size(entry.size))).width(Length::FillPortion(2)),
        container(progress_bar(0.0..=1.0, ratio)).width(Length::FillPortion(3)),
    ]
    .spacing(8)
    .into()
}

pub fn run() -> iced::Result {
    iced::application(GuiApp::default, update, view)
        .title(gui_title)
        .run()
}

fn gui_title(_: &GuiApp) -> String {
    "treefold (GUI)".to_string()
}
