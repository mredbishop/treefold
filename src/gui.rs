use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    BrowsePressed,
    ScanCompleted {
        request_id: u64,
        result: Result<AppState, String>,
        root_input: String,
    },
    ScanProgressTick,
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
    is_scanning: bool,
    scanning_path: Option<String>,
    scanning_current_subfolder: Option<String>,
    progress_shared: Option<Arc<Mutex<Option<String>>>>,
    scan_request_id: u64,
    scan_started_at: Option<Instant>,
    scan_tick: u64,
}

impl Default for GuiApp {
    fn default() -> Self {
        let root = resolve_default_root_path();
        Self::with_root_input(root.display().to_string())
    }
}

impl GuiApp {
    fn with_root_input(root_input: String) -> Self {
        Self {
            root_input,
            state: None,
            error: None,
            selected_heatmap_index: None,
            hovered_heatmap_index: None,
            context_target: None,
            confirm_delete: false,
            is_scanning: false,
            scanning_path: None,
            scanning_current_subfolder: None,
            progress_shared: None,
            scan_request_id: 0,
            scan_started_at: None,
            scan_tick: 0,
        }
    }

    fn apply_loaded_state(&mut self, root_input: String, result: Result<AppState, String>) {
        match result {
            Ok(root) => {
                self.root_input = root_input;
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
        self.is_scanning = false;
        self.scanning_path = None;
        self.scanning_current_subfolder = None;
        self.progress_shared = None;
        self.scan_started_at = None;
        self.scan_tick = 0;
    }
}

pub fn resolve_default_root_path() -> PathBuf {
    resolve_default_root_path_from_env(
        std::env::var_os("HOME").map(PathBuf::from),
        std::env::var_os("USERPROFILE").map(PathBuf::from),
        std::env::current_dir().ok(),
    )
}

pub fn resolve_default_root_path_from_env(
    home: Option<PathBuf>,
    userprofile: Option<PathBuf>,
    cwd: Option<PathBuf>,
) -> PathBuf {
    home.or(userprofile)
        .or(cwd)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn init_state_from_path(path: &str) -> Result<AppState, String> {
    let root = scan_path(&PathBuf::from(path)).map_err(|e| e.to_string())?;
    Ok(AppState::new(root))
}

pub fn update(app: &mut GuiApp, message: Message) -> Task<Message> {
    match message {
        Message::RootChanged(value) => {
            if !app.is_scanning {
                app.root_input = value;
            }
        }
        Message::ScanPressed | Message::RefreshPressed => {
            let path = app.root_input.trim().to_string();
            return start_scan(app, path);
        }
        Message::BrowsePressed => {
            if app.is_scanning {
                return Task::none();
            }
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                return start_scan(app, path.display().to_string());
            }
        }
        Message::ScanCompleted {
            request_id,
            result,
            root_input,
        } => {
            if request_id == app.scan_request_id {
                app.apply_loaded_state(root_input, result);
            }
        }
        Message::ScanProgressTick => {
            app.scan_tick = app.scan_tick.wrapping_add(1);
            if app.is_scanning
                && let Some(shared) = &app.progress_shared
                && let Ok(lock) = shared.lock()
            {
                app.scanning_current_subfolder = lock.clone();
            }
        }
        Message::EnterChild(idx) => {
            if app.is_scanning {
                return Task::none();
            }
            if let Some(state) = app.state.as_mut() {
                state.selected_index = idx;
                state.enter_selected();
                state.clamp_selection();
                app.selected_heatmap_index = Some(state.selected_index);
            }
        }
        Message::HeatmapSelect(idx) => {
            if app.is_scanning {
                return Task::none();
            }
            if let Some(state) = app.state.as_mut() {
                state.selected_index = idx;
                state.clamp_selection();
                app.selected_heatmap_index = Some(state.selected_index);
                app.context_target = None;
                if state
                    .selected_child()
                    .is_some_and(|e| matches!(e.kind, EntryKind::Directory))
                {
                    state.enter_selected();
                }
            }
        }
        Message::GoParent => {
            if app.is_scanning {
                return Task::none();
            }
            if let Some(state) = app.state.as_mut() {
                state.go_parent();
                state.clamp_selection();
                app.selected_heatmap_index = Some(state.selected_index);
            }
        }
        Message::HeatmapEvent(event) => match event {
            HeatmapEvent::Select(idx) => return update(app, Message::HeatmapSelect(idx)),
            HeatmapEvent::Context(idx) => {
                if app.is_scanning {
                    return Task::none();
                }
                app.context_target = Some(idx);
                app.selected_heatmap_index = Some(idx);
                app.confirm_delete = false;
            }
            HeatmapEvent::Hover(idx) => app.hovered_heatmap_index = idx,
        },
        Message::OpenLocation => {
            if app.is_scanning {
                return Task::none();
            }
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
            if app.is_scanning {
                return Task::none();
            }
            if let (Some(state), Some(idx)) = (app.state.as_ref(), app.context_target)
                && let Some(entry) = state.current_children().get(idx)
            {
                if let Err(err) =
                    delete_path(&entry.path, matches!(entry.kind, EntryKind::Directory))
                {
                    app.error = Some(err);
                } else {
                    return start_scan(app, app.root_input.trim().to_string());
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
    let path_input = if app.is_scanning {
        text_input("Path", &app.root_input)
            .padding(8)
            .width(Length::Fill)
    } else {
        text_input("Path", &app.root_input)
            .on_input(Message::RootChanged)
            .on_submit(Message::ScanPressed)
            .padding(8)
            .width(Length::Fill)
    };

    let controls = row![
        path_input,
        if app.is_scanning {
            button("Scan")
        } else {
            button("Scan").on_press(Message::ScanPressed)
        },
        if app.is_scanning {
            button("Browse")
        } else {
            button("Browse").on_press(Message::BrowsePressed)
        },
        if app.is_scanning {
            button("Refresh")
        } else {
            button("Refresh").on_press(Message::RefreshPressed)
        },
        if app.is_scanning {
            button("Up")
        } else {
            button("Up").on_press(Message::GoParent)
        },
    ]
    .spacing(8);

    let loading = if app.is_scanning {
        let spinner = match app.scan_tick % 4 {
            0 => "⠋",
            1 => "⠙",
            2 => "⠹",
            _ => "⠸",
        };
        let elapsed = app
            .scan_started_at
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);
        text(format!(
            "{spinner} Scanning: {}{} | {}s elapsed",
            app.scanning_path.as_deref().unwrap_or("<unknown>"),
            app.scanning_current_subfolder
                .as_ref()
                .map(|p| format!(" | current: {p}"))
                .unwrap_or_else(|| " | current: preparing...".to_string()),
            elapsed
        ))
    } else {
        text("")
    };

    let body: Element<'_, Message> = if app.is_scanning && app.state.is_none() {
        let active = app
            .scanning_current_subfolder
            .as_deref()
            .unwrap_or("Preparing scan...");
        container(
            column![
                text("Scanning in progress").size(24),
                text(format!(
                    "Root: {}",
                    app.scanning_path.as_deref().unwrap_or("<unknown>")
                )),
                text(format!("Current: {active}")),
                text("Waiting for directory access prompts is normal on macOS.")
            ]
            .spacing(8),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else if let Some(state) = &app.state {
        let children = state.current_children();
        let mut list_col =
            column![text(format!("Current: {}", state.current_dir.display()))].spacing(6);
        for (idx, child) in children.iter().enumerate() {
            let kind = match child.kind {
                EntryKind::Directory => "d",
                EntryKind::File => "f",
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
        container(text("Loading..."))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    let mut root = column![controls, loading, body].spacing(10).padding(10);
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
    let initial_root = path
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| resolve_default_root_path().display().to_string());

    iced::application(
        move || {
            let mut app = GuiApp::with_root_input(initial_root.clone());
            let task = start_scan(&mut app, initial_root.clone());
            (app, task)
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
    Subscription::batch(vec![
        event::listen().map(Message::EventOccurred),
        iced::time::every(Duration::from_millis(120)).map(|_| Message::ScanProgressTick),
    ])
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
    if app.is_scanning {
        return;
    }

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

fn start_scan(app: &mut GuiApp, path: String) -> Task<Message> {
    app.scan_request_id = app.scan_request_id.saturating_add(1);
    let request_id = app.scan_request_id;
    app.is_scanning = true;
    app.scanning_path = Some(path.clone());
    app.scanning_current_subfolder = None;
    app.scan_started_at = Some(Instant::now());
    app.scan_tick = 0;
    let shared_progress: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    app.progress_shared = Some(shared_progress.clone());
    let run_path = path.clone();
    let done_path = path;
    let progress_sink = shared_progress;

    Task::perform(
        async move {
            let mut cb = |p: &std::path::Path| {
                if let Ok(mut lock) = progress_sink.lock() {
                    *lock = Some(p.display().to_string());
                }
            };
            crate::fs_scan::scan_path_with_progress(std::path::Path::new(&run_path), &mut cb)
                .map(AppState::new)
                .map_err(|e| e.to_string())
        },
        move |result| Message::ScanCompleted {
            request_id,
            result,
            root_input: done_path.clone(),
        },
    )
}
