use std::path::PathBuf;

use iced::event;
use iced::keyboard::{Event as KeyEvent, Key, key::Named};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Element, Length, Subscription, Task};

use crate::fs_scan::{EntryKind, scan_path};
use crate::gui_heatmap::{HeatmapEvent, heatmap_canvas};
use crate::os_actions::{delete_path, open_in_file_browser};
use crate::state::AppState;

#[derive(Debug, Clone)]
pub enum Message {
    RootChanged(String),
    ScanPressed,
    RefreshPressed,
    EnterChild(usize),
    HeatmapSelect(usize),
    HeatmapEvent(HeatmapEvent),
    GoParent,
    EventOccurred(iced::Event),
    OpenLocation,
    RequestDelete,
    ConfirmDelete,
    CancelDelete,
    ClearContext,
}

pub struct GuiApp {
    root_input: String,
    state: Option<AppState>,
    error: Option<String>,
    selected_heatmap_index: Option<usize>,
    hovered_heatmap_index: Option<usize>,
    context_target: Option<usize>,
    confirm_delete: bool,
}

impl Default for GuiApp {
    fn default() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::with_root_input(cwd.display().to_string())
    }
}

impl GuiApp {
    fn with_root_input(root_input: String) -> Self {
        let mut app = Self {
            root_input,
            state: None,
            error: None,
            selected_heatmap_index: None,
            hovered_heatmap_index: None,
            context_target: None,
            confirm_delete: false,
        };
        app.scan_current_root();
        app
    }

    fn scan_current_root(&mut self) {
        match init_state_from_path(self.root_input.trim()) {
            Ok(root) => {
                self.state = Some(root);
                self.error = None;
                self.selected_heatmap_index = Some(0);
                self.hovered_heatmap_index = None;
                self.context_target = None;
                self.confirm_delete = false;
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
                state.clamp_selection();
                app.selected_heatmap_index = Some(state.selected_index);
            }
        }
        Message::HeatmapSelect(idx) => {
            if let Some(state) = app.state.as_mut() {
                state.selected_index = idx;
                state.clamp_selection();
                app.selected_heatmap_index = Some(state.selected_index);
                app.context_target = None;
                if state
                    .selected_child()
                    .is_some_and(|e| matches!(e.kind, crate::fs_scan::EntryKind::Directory))
                {
                    state.enter_selected();
                }
            }
        }
        Message::GoParent => {
            if let Some(state) = app.state.as_mut() {
                state.go_parent();
                state.clamp_selection();
                app.selected_heatmap_index = Some(state.selected_index);
            }
        }
        Message::HeatmapEvent(event) => match event {
            HeatmapEvent::Select(idx) => return update(app, Message::HeatmapSelect(idx)),
            HeatmapEvent::Context(idx) => {
                app.context_target = Some(idx);
                app.selected_heatmap_index = Some(idx);
                app.confirm_delete = false;
            }
            HeatmapEvent::Hover(idx) => app.hovered_heatmap_index = idx,
        },
        Message::OpenLocation => {
            if let (Some(state), Some(idx)) = (app.state.as_ref(), app.context_target)
                && let Some(entry) = state.current_children().get(idx)
            {
                let target = open_target_path(entry.kind, &entry.path);
                if let Err(err) = open_in_file_browser(&target) {
                    app.error = Some(err);
                }
            }
            app.context_target = None;
        }
        Message::RequestDelete => app.confirm_delete = true,
        Message::ConfirmDelete => {
            if let (Some(state), Some(idx)) = (app.state.as_ref(), app.context_target)
                && let Some(entry) = state.current_children().get(idx)
            {
                if let Err(err) =
                    delete_path(&entry.path, matches!(entry.kind, EntryKind::Directory))
                {
                    app.error = Some(err);
                } else {
                    app.scan_current_root();
                }
            }
            app.context_target = None;
            app.confirm_delete = false;
        }
        Message::CancelDelete => app.confirm_delete = false,
        Message::ClearContext => {
            app.context_target = None;
            app.confirm_delete = false;
        }
        Message::EventOccurred(event) => handle_gui_event(app, event),
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
                    crate::layout::human_size(child.size)
                )))
                .on_press(Message::EnterChild(idx))
                .style(if state.selected_index == idx {
                    iced::widget::button::success
                } else {
                    iced::widget::button::secondary
                })
                .width(Length::Fill),
            );
        }
        let left = scrollable(list_col).height(Length::Fill);

        let selected_info = state
            .selected_child()
            .map(|e| {
                format!(
                    "Selected: {} | {} | {}",
                    e.name,
                    crate::layout::human_size(e.size),
                    e.path.display()
                )
            })
            .unwrap_or_else(|| "Selected: none".to_string());
        let hover_info = app
            .hovered_heatmap_index
            .and_then(|idx| state.current_children().get(idx))
            .map(|e| {
                format!(
                    "Hover: {} | {} | {} | {}",
                    e.name,
                    if matches!(e.kind, EntryKind::Directory) {
                        "folder"
                    } else {
                        "file"
                    },
                    crate::layout::human_size(e.size),
                    e.path.display()
                )
            })
            .unwrap_or_else(|| "Hover: none".to_string());
        let right = column![
            text("Heatmap (SequoiaView style)"),
            container(heatmap_canvas(
                children.to_vec(),
                app.selected_heatmap_index,
                app.hovered_heatmap_index,
                Message::HeatmapEvent
            ))
            .width(Length::Fill)
            .height(Length::Fill),
            text(selected_info),
            text(hover_info),
            context_menu_view(app, state)
        ]
        .height(Length::Fill)
        .spacing(8);
        row![
            container(left).width(Length::FillPortion(1)),
            container(right)
                .width(Length::FillPortion(2))
                .height(Length::Fill),
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

fn context_menu_view<'a>(app: &'a GuiApp, state: &'a AppState) -> Element<'a, Message> {
    let Some(idx) = app.context_target else {
        return container(text("")).into();
    };
    let Some(entry) = state.current_children().get(idx) else {
        return container(text("")).into();
    };

    let open_label = context_open_label(entry.kind);
    let delete_label = context_delete_label(entry.kind);
    let mut col = column![
        text(format!("Context: {}", entry.path.display())),
        row![
            button(open_label).on_press(Message::OpenLocation),
            button(delete_label).on_press(Message::RequestDelete),
            button("Close").on_press(Message::ClearContext),
        ]
        .spacing(8)
    ]
    .spacing(6);
    if app.confirm_delete {
        col = col.push(text("Confirm delete? This cannot be undone."));
        col = col.push(
            row![
                button("Confirm delete").on_press(Message::ConfirmDelete),
                button("Cancel").on_press(Message::CancelDelete)
            ]
            .spacing(8),
        );
    }
    container(col).into()
}

pub fn run() -> iced::Result {
    run_with_path(None)
}

pub fn run_with_path(path: Option<PathBuf>) -> iced::Result {
    iced::application(
        move || {
            if let Some(path) = path.clone() {
                GuiApp::with_root_input(path.display().to_string())
            } else {
                GuiApp::default()
            }
        },
        update,
        view,
    )
    .title(gui_title)
    .subscription(subscription)
    .run()
}

fn gui_title(_: &GuiApp) -> String {
    "treefold (GUI)".to_string()
}

fn subscription(_state: &GuiApp) -> Subscription<Message> {
    event::listen().map(Message::EventOccurred)
}

pub fn context_open_label(kind: EntryKind) -> &'static str {
    if matches!(kind, EntryKind::Directory) {
        "Open this directory"
    } else {
        "View in parent"
    }
}

pub fn context_delete_label(kind: EntryKind) -> &'static str {
    if matches!(kind, EntryKind::Directory) {
        "Delete this folder"
    } else {
        "Delete this file"
    }
}

pub fn open_target_path(kind: EntryKind, path: &std::path::Path) -> std::path::PathBuf {
    if matches!(kind, EntryKind::Directory) {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiKeyAction {
    Up,
    Down,
    Enter,
    Back,
    None,
}

pub fn map_key_event(event: &iced::Event) -> GuiKeyAction {
    if let iced::Event::Keyboard(KeyEvent::KeyPressed { key, .. }) = event {
        return match key.as_ref() {
            Key::Named(Named::ArrowUp) => GuiKeyAction::Up,
            Key::Named(Named::ArrowDown) => GuiKeyAction::Down,
            Key::Named(Named::ArrowRight) | Key::Named(Named::Enter) => GuiKeyAction::Enter,
            Key::Named(Named::ArrowLeft) | Key::Named(Named::Escape) => GuiKeyAction::Back,
            _ => GuiKeyAction::None,
        };
    }
    GuiKeyAction::None
}

fn handle_gui_event(app: &mut GuiApp, event: iced::Event) {
    let action = map_key_event(&event);
    let Some(state) = app.state.as_mut() else {
        return;
    };
    match action {
        GuiKeyAction::Up => state.move_up(),
        GuiKeyAction::Down => state.move_down(),
        GuiKeyAction::Enter => {
            let before = state.current_dir.clone();
            state.enter_selected();
            if state.current_dir != before {
                state.selected_index = 0;
            }
        }
        GuiKeyAction::Back => state.go_parent(),
        GuiKeyAction::None => {}
    }
    state.clamp_selection();
    app.selected_heatmap_index = Some(state.selected_index);
}
