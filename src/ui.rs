use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{io::Result, sync::mpsc};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame, Terminal,
};

use crate::{
    app::{App, DetailViewState, Notification, ViewState},
    event::AppEventType,
    item::{FileDetail, FileVersion, Item},
    key_code, key_code_char,
};

const APP_NAME: &str = "STU";

const SELECTED_COLOR: Color = Color::Cyan;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const APP_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

pub async fn run<B: Backend>(
    app: &mut App,
    terminal: &mut Terminal<B>,
    rx: mpsc::Receiver<AppEventType>,
) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;
        match rx.recv().unwrap() {
            AppEventType::Key(key) => {
                match key {
                    key_code!(KeyCode::Esc) | key_code_char!('c', Ctrl) => return Ok(()),
                    _ => {}
                }

                if app.app_view_state.is_loading {
                    continue;
                }

                match app.app_view_state.notification {
                    Notification::Error(_) => {
                        if app.app_view_state.view_state == ViewState::Initializing {
                            return Ok(());
                        }
                        app.app_view_state.notification = Notification::None;
                        continue;
                    }
                    Notification::Info(_) => {
                        app.app_view_state.notification = Notification::None;
                    }
                    Notification::None => {}
                }

                match key {
                    key_code_char!('j') => {
                        app.select_next();
                    }
                    key_code_char!('k') => {
                        app.select_prev();
                    }
                    key_code_char!('g') => {
                        app.select_first();
                    }
                    key_code_char!('G') => {
                        app.select_last();
                    }
                    key_code_char!('f') => {
                        app.select_next_page();
                    }
                    key_code_char!('b') => {
                        app.select_prev_page();
                    }
                    key_code!(KeyCode::Enter) | key_code_char!('m', Ctrl) => {
                        app.move_down();
                    }
                    key_code!(KeyCode::Backspace) | key_code_char!('h', Ctrl) => {
                        app.move_up();
                    }
                    key_code_char!('h') | key_code_char!('l') => {
                        app.select_tabs();
                    }
                    key_code_char!('s') => {
                        app.download();
                    }
                    key_code_char!('x') => {
                        app.open_management_console();
                    }
                    key_code_char!('?') => {
                        app.toggle_help();
                    }
                    _ => {}
                }
            }
            AppEventType::Resize(_, height) => {
                app.resize(height as usize);
            }
            AppEventType::Initialize(config, client) => {
                app.initialize(config, client).await;
            }
            AppEventType::LoadObjects => {
                app.load_objects();
            }
            AppEventType::CompleteLoadObjects(result) => {
                app.complete_load_objects(result);
            }
            AppEventType::LoadObject => {
                app.load_object();
            }
            AppEventType::CompleteLoadObject(result) => {
                app.complete_load_object(result);
            }
            AppEventType::DownloadObject => {
                app.download_object();
            }
            AppEventType::CompleteDownloadObject(result) => {
                app.complete_download_object(result);
            }
            AppEventType::Info(msg) => {
                app.app_view_state.notification = Notification::Info(msg);
            }
            AppEventType::Error(e) => {
                app.save_error(&e);
                app.app_view_state.notification = Notification::Error(e.msg);
            }
        }
    }
}

fn render<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(f.size());

    render_content(f, chunks[0], app);
    render_footer(f, chunks[1], app);
    render_loading_dialog(f, app);
}

fn render_content<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    match app.app_view_state.view_state {
        ViewState::Initializing => render_initializing_view(f, area, app),
        ViewState::List => render_list_view(f, area, app),
        ViewState::Detail(vs) => render_detail_view(f, area, app, &vs),
        ViewState::Help => render_help_view(f, area, app),
    }
}

fn render_initializing_view<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let header = build_header(app);
    f.render_widget(header, chunks[0]);

    let content = Block::default().borders(Borders::all());
    f.render_widget(content, chunks[1]);
}

fn render_list_view<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let header = build_header(app);
    f.render_widget(header, chunks[0]);

    let current_items = app.current_items();
    let current_selected = app.app_view_state.list_state.selected;
    let current_offset = app.app_view_state.list_state.offset;
    let list = build_list(
        &current_items,
        current_selected,
        current_offset,
        f.size().width,
        chunks[1].height,
        SELECTED_COLOR,
    );
    f.render_widget(list, chunks[1]);
}

fn render_detail_view<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App, vs: &DetailViewState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let header = build_header(app);
    f.render_widget(header, chunks[0]);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let current_items = app.current_items();
    let current_selected = app.app_view_state.list_state.selected;
    let current_offset = app.app_view_state.list_state.offset;
    let list = build_list(
        &current_items,
        current_selected,
        current_offset,
        f.size().width,
        chunks[0].height,
        Color::DarkGray,
    );
    f.render_widget(list, chunks[0]);

    let block = build_file_detail_block("");
    f.render_widget(block, chunks[1]);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)].as_ref())
        .margin(1)
        .split(chunks[1]);

    let tabs = build_file_detail_tabs(vs);
    f.render_widget(tabs, chunks[0]);

    match vs {
        DetailViewState::Detail => {
            let current_file_detail = app.get_current_file_detail().unwrap();
            let detail = build_file_detail(current_file_detail);
            f.render_widget(detail, chunks[1]);
        }
        DetailViewState::Version => {
            let current_file_versions = app.get_current_file_versions().unwrap();
            let versions = build_file_versions(current_file_versions, chunks[1].width);
            f.render_widget(versions, chunks[1]);
        }
    }
}

fn render_help_view<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let content = build_help(app, f.size().width);
    f.render_widget(content, area);
}

fn build_header(app: &App) -> Paragraph {
    let current_key = app.current_key_string();
    Paragraph::new(Span::styled(current_key, Style::default())).block(
        Block::default()
            .title(Span::styled(APP_NAME, Style::default()))
            .borders(Borders::all()),
    )
}

fn build_list(
    current_items: &[Item],
    current_selected: usize,
    current_offset: usize,
    width: u16,
    height: u16,
    color: Color,
) -> List {
    let show_item_count = (height as usize) - 2 /* border */;
    let list_items: Vec<ListItem> = current_items
        .iter()
        .skip(current_offset)
        .take(show_item_count)
        .enumerate()
        .map(|(i, item)| {
            let content = match item {
                Item::Bucket { name, .. } => {
                    let content = format_bucket_item(name, width);
                    let style = Style::default();
                    Span::styled(content, style)
                }
                Item::Dir { name, .. } => {
                    let content = format_dir_item(name, width);
                    let style = Style::default().add_modifier(Modifier::BOLD);
                    Span::styled(content, style)
                }
                Item::File {
                    name,
                    size_byte,
                    last_modified,
                    ..
                } => {
                    let content = format_file_item(name, size_byte, last_modified, width);
                    let style = Style::default();
                    Span::styled(content, style)
                }
            };
            if i + current_offset == current_selected {
                ListItem::new(content).style(Style::default().bg(color))
            } else {
                ListItem::new(content)
            }
        })
        .collect();

    let title = format_list_count(current_items, current_selected);
    List::new(list_items).block(
        Block::default()
            .borders(Borders::all())
            .title(title)
            .title_alignment(Alignment::Right),
    )
}

fn format_bucket_item(name: &String, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn format_dir_item(name: &String, width: u16) -> String {
    let name_w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;
    let name = format!("{}/", name);
    format!(" {:<name_w$} ", name, name_w = name_w)
}

fn format_file_item(
    name: &String,
    size_byte: &i64,
    last_modified: &DateTime<Local>,
    width: u16,
) -> String {
    let size = format_size_byte(*size_byte);
    let date = format_datetime(last_modified);
    let date_w: usize = 17;
    let size_w: usize = 10;
    let name_w: usize = (width as usize) - date_w - size_w - 12 /* spaces */ - 2 /* border */;
    format!(
        " {:<name_w$}    {:<date_w$}    {:<size_w$} ",
        name,
        date,
        size,
        name_w = name_w,
        date_w = date_w,
        size_w = size_w
    )
}

fn format_list_count(current_items: &[Item], current_selected: usize) -> String {
    let total = current_items.len();
    if total == 0 {
        String::new()
    } else {
        format_count(current_selected + 1, total)
    }
}

fn format_count(selected: usize, total: usize) -> String {
    let digits = digits(total);
    format!(" {:>digits$} / {} ", selected, total)
}

fn digits(n: usize) -> usize {
    n.to_string().len()
}

fn build_file_detail_block(title: &str) -> Block {
    Block::default().title(title).borders(Borders::all())
}

fn build_file_detail_tabs(selected: &DetailViewState) -> Tabs {
    let tabs = vec![
        Spans::from(Span::styled("Detail", Style::default())),
        Spans::from(Span::styled("Version", Style::default())),
    ];
    Tabs::new(tabs)
        .select(*selected as usize)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(SELECTED_COLOR),
        )
        .block(Block::default().borders(Borders::BOTTOM))
}

fn build_file_detail(detail: &FileDetail) -> Paragraph {
    let text = vec![
        Spans::from(Span::styled(
            " Name:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", &detail.name),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            " Size:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", format_size_byte(detail.size_byte)),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            " Last Modified:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", format_datetime(&detail.last_modified)),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            " ETag:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", &detail.e_tag),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            " Content-Type:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Spans::from(Span::styled(
            format!("  {}", &detail.content_type),
            Style::default(),
        )),
    ];
    Paragraph::new(text).block(Block::default())
}

fn format_size_byte(size_byte: i64) -> String {
    humansize::format_size_i(size_byte, humansize::BINARY)
}

fn format_datetime(datetime: &DateTime<Local>) -> String {
    datetime.format("%y/%m/%d %H:%M:%S").to_string()
}

fn build_file_versions(versions: &[FileVersion], width: u16) -> List {
    let list_items: Vec<ListItem> = versions
        .iter()
        .map(|v| {
            let content = vec![
                Spans::from(vec![
                    Span::styled(
                        "    Version ID: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&v.version_id, Style::default()),
                ]),
                Spans::from(vec![
                    Span::styled(
                        " Last Modified: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format_datetime(&v.last_modified), Style::default()),
                ]),
                Spans::from(vec![
                    Span::styled(
                        "          Size: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format_size_byte(v.size_byte), Style::default()),
                ]),
                Spans::from(Span::styled(
                    "-".repeat(width as usize),
                    Style::default().fg(Color::DarkGray),
                )),
            ];
            ListItem::new(content)
        })
        .collect();
    List::new(list_items)
        .block(Block::default())
        .highlight_style(Style::default().bg(SELECTED_COLOR))
}

fn build_help(app: &App, width: u16) -> Paragraph {
    let w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;

    let app_detail = vec![
        Spans::from(""),
        Spans::from(Span::styled(
            format!("  {} - {}", APP_NAME, APP_DESCRIPTION),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            format!("  Version: {}", APP_VERSION),
            Style::default(),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            format!("  {}", APP_HOMEPAGE),
            Style::default().fg(Color::Blue),
        )),
        Spans::from(""),
        Spans::from(Span::styled(
            format!(" {}", "-".repeat(w)),
            Style::default().fg(Color::DarkGray),
        )),
        Spans::from(""),
    ]
    .into_iter();

    let help = match app.app_view_state.before_view_state {
        Some(vs) => match vs {
            ViewState::Initializing | ViewState::Help => vec![],
            ViewState::List => {
                vec![
                    Spans::from(Span::styled(
                        "  <Esc> <Ctrl-c>: Quit app,  <j/k>: Select item,  <g/G>: Go to top/bottom",
                        Style::default(),
                    )),
                    Spans::from(""),
                    Spans::from(Span::styled(
                        "  <f>: Scroll page forward,  <b>: Scroll page backward",
                        Style::default(),
                    )),
                    Spans::from(""),
                    Spans::from(Span::styled(
                        "  <Enter>: Open file or folder,  <Backspace>: Go back to prev folder",
                        Style::default(),
                    )),
                    Spans::from(""),
                    Spans::from(Span::styled(
                        "  <x>: Open management console in browser",
                        Style::default(),
                    )),
                ]
            }
            ViewState::Detail(_) => {
                vec![
                    Spans::from(Span::styled(
                        "  <Esc> <Ctrl-c>: Quit app,  <h/l>: Select tabs,  <Backspace>: Close detail panel",
                        Style::default(),
                    )),
                    Spans::from(""),
                    Spans::from(Span::styled(
                        "  <s>: Download object,  <x>: Open management console in browser",
                        Style::default(),
                    )),
                ]
            }
        },
        None => vec![],
    }
    .into_iter();

    let content: Vec<Spans> = app_detail.chain(help).collect();
    Paragraph::new(content).block(Block::default().title(APP_NAME).borders(Borders::all()))
}

fn build_short_help(app: &App) -> Paragraph {
    let help = match app.app_view_state.view_state {
        ViewState::Initializing => "",
        ViewState::List => {
            "<Esc>: Quit, <j/k>: Select, <Enter>: Open, <Backspace>: Go back, <?> Help"
        }
        ViewState::Detail(_) => {
            "<Esc>: Quit, <h/l>: Select tabs, <s>: Download, <Backspace>: Close, <?> Help"
        }
        ViewState::Help => "<Esc>: Quit, <?>: Close help",
    };
    let help = format!("  {}", help);
    Paragraph::new(Span::styled(help, Style::default().fg(Color::DarkGray))).block(Block::default())
}

fn build_info_status(msg: &String) -> Paragraph {
    let msg = format!("  {}", msg);
    Paragraph::new(Span::styled(
        msg,
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Green),
    ))
    .block(Block::default())
}

fn build_error_status(err: &String) -> Paragraph {
    let err = format!("  ERROR: {}", err);
    Paragraph::new(Span::styled(
        err,
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Red),
    ))
    .block(Block::default())
}

fn render_footer<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    match &app.app_view_state.notification {
        Notification::Info(msg) => {
            let msg = build_info_status(msg);
            f.render_widget(msg, area);
        }
        Notification::Error(msg) => {
            let msg = build_error_status(msg);
            f.render_widget(msg, area);
        }
        Notification::None => {
            let help = build_short_help(app);
            f.render_widget(help, area);
        }
    }
}

fn build_loading_dialog(msg: &str) -> Paragraph {
    let text = vec![
        Spans::from(""),
        Spans::from(Span::styled(
            msg,
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];
    Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::all())
            .border_type(BorderType::Double),
    )
}

fn loading_dialog_rect(r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((r.height - 5) / 2),
                Constraint::Length(5),
                Constraint::Length((r.height - 5) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - 30) / 2),
                Constraint::Length(30),
                Constraint::Length((r.width - 30) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn render_loading_dialog<B: Backend>(f: &mut Frame<B>, app: &App) {
    if app.app_view_state.is_loading {
        let loading = build_loading_dialog("Loading...");
        let area = loading_dialog_rect(f.size());
        f.render_widget(Clear, area);
        f.render_widget(loading, area);
    }
}
