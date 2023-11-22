use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Margin,
    style::{Color, Modifier, Style},
    symbols::scrollbar::VERTICAL,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Scrollbar,
        ScrollbarState, Tabs,
    },
    Frame, Terminal,
};
use std::{io::Result, sync::mpsc};

use crate::{
    app::{App, DetailViewState, Notification, ViewState},
    event::AppEventType,
    item::{BucketItem, FileDetail, FileVersion, ObjectItem},
    key_code, key_code_char,
};

const APP_NAME: &str = "STU";

const SELECTED_COLOR: Color = Color::Cyan;
const SELECTED_DISABLED_COLOR: Color = Color::DarkGray;
const SELECTED_ITEM_TEXT_COLOR: Color = Color::Black;
const DIVIDER_COLOR: Color = Color::DarkGray;
const SCROLLBAR_COLOR: Color = Color::DarkGray;
const LINK_TEXT_COLOR: Color = Color::Blue;
const SHORT_HELP_COLOR: Color = Color::DarkGray;
const INFO_STATUS_COLOR: Color = Color::Green;
const ERROR_STATUS_COLOR: Color = Color::Red;

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
                    key_code!(KeyCode::Esc) | key_code_char!('c', Ctrl) => {
                        return Ok(());
                    }
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
                    key_code_char!('~') => {
                        app.back_to_bucket_list();
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
                app.initialize(config, client);
            }
            AppEventType::CompleteInitialize(result) => {
                app.complete_initialize(result);
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

fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)].as_ref())
        .split(f.size());

    render_content(f, chunks[0], app);
    render_footer(f, chunks[1], app);
    render_loading_dialog(f, app);
}

fn render_content(f: &mut Frame, area: Rect, app: &App) {
    match &app.app_view_state.view_state {
        ViewState::Initializing => render_initializing_view(f, area, app),
        ViewState::BucketList => render_bucket_list_view(f, area, app),
        ViewState::ObjectList => render_object_list_view(f, area, app),
        ViewState::Detail(vs) => render_detail_view(f, area, app, vs),
        ViewState::Help(before) => render_help_view(f, area, before),
    }
}

fn render_initializing_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let header = build_header(app);
    f.render_widget(header, chunks[0]);

    let content = Block::default().borders(Borders::all());
    f.render_widget(content, chunks[1]);
}

fn render_bucket_list_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let header = build_header(app);
    f.render_widget(header, chunks[0]);

    let current_items = app.bucket_items();
    let list_state = ListViewState {
        current_selected: app.app_view_state.list_state.selected,
        current_offset: app.app_view_state.list_state.offset,
    };
    let styles = ListItemStyles {
        selected_bg_color: SELECTED_COLOR,
        selected_fg_color: SELECTED_ITEM_TEXT_COLOR,
    };
    let list_items =
        build_list_items_from_bucket_items(&current_items, list_state, chunks[1], styles);
    let list = build_list(list_items, current_items.len(), list_state.current_selected);
    f.render_widget(list, chunks[1]);

    render_list_scroll_bar(f, chunks[1], list_state, current_items.len());
}

fn render_object_list_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(area);

    let header = build_header(app);
    f.render_widget(header, chunks[0]);

    let current_items = app.current_object_items();
    let list_state = ListViewState {
        current_selected: app.app_view_state.list_state.selected,
        current_offset: app.app_view_state.list_state.offset,
    };
    let styles = ListItemStyles {
        selected_bg_color: SELECTED_COLOR,
        selected_fg_color: SELECTED_ITEM_TEXT_COLOR,
    };
    let list_items =
        build_list_items_from_object_items(&current_items, list_state, chunks[1], styles, true);
    let list = build_list(list_items, current_items.len(), list_state.current_selected);
    f.render_widget(list, chunks[1]);

    render_list_scroll_bar(f, chunks[1], list_state, current_items.len());
}

fn render_list_scroll_bar(
    f: &mut Frame,
    area: Rect,
    list_state: ListViewState,
    current_items_len: usize,
) {
    // fixme:
    //  It seems to be better to calculate based on offset position
    //  Scrolling with f/b has been implemented by extending the list independently,
    //  but this does not seem to be suitable for standard scrolling behavior

    let current_selected = list_state.current_selected;
    let scrollbar_area = area.inner(&Margin {
        horizontal: 1,
        vertical: 1,
    });
    let scrollbar_area_height = scrollbar_area.height as usize;
    if current_items_len > scrollbar_area_height {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(current_items_len)
            .viewport_content_length(scrollbar_area_height)
            .position(current_selected);
        let scrollbar = Scrollbar::default()
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some(VERTICAL.track))
            .track_style(Style::default().fg(SCROLLBAR_COLOR))
            .thumb_symbol(VERTICAL.thumb)
            .thumb_style(Style::default().fg(SCROLLBAR_COLOR));
        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state)
    }
}

fn render_detail_view(f: &mut Frame, area: Rect, app: &App, vs: &DetailViewState) {
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

    let current_items = app.current_object_items();
    let list_state = ListViewState {
        current_selected: app.app_view_state.list_state.selected,
        current_offset: app.app_view_state.list_state.offset,
    };
    let styles = ListItemStyles {
        selected_bg_color: SELECTED_DISABLED_COLOR,
        selected_fg_color: SELECTED_ITEM_TEXT_COLOR,
    };
    let list_items =
        build_list_items_from_object_items(&current_items, list_state, chunks[0], styles, false);
    let list = build_list(list_items, current_items.len(), list_state.current_selected);
    f.render_widget(list, chunks[0]);

    let block = build_file_detail_block();
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

fn render_help_view(f: &mut Frame, area: Rect, before: &ViewState) {
    let content = build_help(before, f.size().width);
    f.render_widget(content, area);
}

fn build_header(app: &App) -> Paragraph {
    let current_key = app.object_key_breadcrumb_string();
    Paragraph::new(Span::styled(current_key, Style::default())).block(
        Block::default()
            .title(Span::styled(APP_NAME, Style::default()))
            .padding(Padding::horizontal(1))
            .borders(Borders::all()),
    )
}

fn build_list(list_items: Vec<ListItem>, total_count: usize, current_selected: usize) -> List {
    let title = format_list_count(total_count, current_selected);
    List::new(list_items).block(
        Block::default()
            .title(title)
            .title_alignment(Alignment::Right)
            .padding(Padding::horizontal(1))
            .borders(Borders::all()),
    )
}

#[derive(Clone, Copy, Debug)]
struct ListViewState {
    current_selected: usize,
    current_offset: usize,
}

#[derive(Clone, Copy, Debug)]
struct ListItemStyles {
    selected_bg_color: Color,
    selected_fg_color: Color,
}

fn build_list_items_from_bucket_items(
    current_items: &[BucketItem],
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
) -> Vec<ListItem> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    current_items
        .iter()
        .skip(list_state.current_offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| build_list_item_from_bucket_item(idx, item, list_state, area, styles))
        .collect()
}

fn build_list_item_from_bucket_item(
    idx: usize,
    item: &BucketItem,
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
) -> ListItem {
    let content = format_bucket_item(&item.name, area.width);
    let style = Style::default();
    let span = Span::styled(content, style);
    if idx + list_state.current_offset == list_state.current_selected {
        ListItem::new(span).style(
            Style::default()
                .bg(styles.selected_bg_color)
                .fg(styles.selected_fg_color),
        )
    } else {
        ListItem::new(span)
    }
}

fn build_list_items_from_object_items(
    current_items: &[ObjectItem],
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
    show_file_detail: bool,
) -> Vec<ListItem> {
    let show_item_count = (area.height as usize) - 2 /* border */;
    current_items
        .iter()
        .skip(list_state.current_offset)
        .take(show_item_count)
        .enumerate()
        .map(|(idx, item)| {
            build_list_item_from_object_item(idx, item, list_state, area, styles, show_file_detail)
        })
        .collect()
}

fn build_list_item_from_object_item(
    idx: usize,
    item: &ObjectItem,
    list_state: ListViewState,
    area: Rect,
    styles: ListItemStyles,
    show_file_detail: bool,
) -> ListItem {
    let content = match item {
        ObjectItem::Dir { name, .. } => {
            let content = format_dir_item(name, area.width);
            let style = Style::default().add_modifier(Modifier::BOLD);
            Span::styled(content, style)
        }
        ObjectItem::File {
            name,
            size_byte,
            last_modified,
            ..
        } => {
            let content =
                format_file_item(name, size_byte, last_modified, area.width, show_file_detail);
            let style = Style::default();
            Span::styled(content, style)
        }
    };
    if idx + list_state.current_offset == list_state.current_selected {
        ListItem::new(content).style(
            Style::default()
                .bg(styles.selected_bg_color)
                .fg(styles.selected_fg_color),
        )
    } else {
        ListItem::new(content)
    }
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
    show_file_detail: bool,
) -> String {
    if show_file_detail {
        let size = format_size_byte(*size_byte);
        let date = format_datetime(last_modified);
        let date_w: usize = 19;
        let size_w: usize = 10;
        let name_w: usize = (width as usize) - date_w - size_w - 10 /* spaces */ - 4 /* border + space */;
        format!(
            " {:<name_w$}    {:<date_w$}    {:>size_w$} ",
            name,
            date,
            size,
            name_w = name_w,
            date_w = date_w,
            size_w = size_w
        )
    } else {
        let name_w: usize = (width as usize) - 2 /* spaces */ - 4 /* border */;
        format!(" {:<name_w$} ", name, name_w = name_w)
    }
}

fn format_list_count(total_count: usize, current_selected: usize) -> String {
    if total_count == 0 {
        String::new()
    } else {
        format_count(current_selected + 1, total_count)
    }
}

fn format_count(selected: usize, total: usize) -> String {
    let digits = digits(total);
    format!(" {:>digits$} / {} ", selected, total)
}

fn digits(n: usize) -> usize {
    n.to_string().len()
}

fn build_file_detail_block() -> Block<'static> {
    Block::default().borders(Borders::all())
}

fn build_file_detail_tabs(selected: &DetailViewState) -> Tabs {
    let tabs = vec![
        Line::from(Span::styled("Detail", Style::default())),
        Line::from(Span::styled("Version", Style::default())),
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
        Line::from(Span::styled(
            " Name:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", &detail.name),
            Style::default(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Size:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", format_size_byte(detail.size_byte)),
            Style::default(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Last Modified:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", format_datetime(&detail.last_modified)),
            Style::default(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " ETag:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", &detail.e_tag),
            Style::default(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Content-Type:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
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
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn build_file_versions(versions: &[FileVersion], width: u16) -> List {
    let list_items: Vec<ListItem> = versions
        .iter()
        .map(|v| {
            let content = vec![
                Line::from(vec![
                    Span::styled(
                        "    Version ID: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&v.version_id, Style::default()),
                ]),
                Line::from(vec![
                    Span::styled(
                        " Last Modified: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format_datetime(&v.last_modified), Style::default()),
                ]),
                Line::from(vec![
                    Span::styled(
                        "          Size: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format_size_byte(v.size_byte), Style::default()),
                ]),
                Line::from(Span::styled(
                    "-".repeat(width as usize),
                    Style::default().fg(DIVIDER_COLOR),
                )),
            ];
            ListItem::new(content)
        })
        .collect();
    List::new(list_items)
        .block(Block::default())
        .highlight_style(Style::default().bg(SELECTED_COLOR))
}

fn build_help(before: &ViewState, width: u16) -> Paragraph<'static> {
    let w: usize = (width as usize) - 2 /* spaces */ - 2 /* border */;

    let app_detail = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} - {}", APP_NAME, APP_DESCRIPTION),
            Style::default(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  Version: {}", APP_VERSION),
            Style::default(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", APP_HOMEPAGE),
            Style::default().fg(LINK_TEXT_COLOR),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(" {}", "-".repeat(w)),
            Style::default().fg(DIVIDER_COLOR),
        )),
        Line::from(""),
    ]
    .into_iter();

    let help = match before {
            ViewState::Initializing | ViewState::Help(_) => vec![],
            ViewState::BucketList => {
                vec![
                    Line::from(Span::styled(
                        "  <Esc> <Ctrl-c>: Quit app,  <j/k>: Select item,  <g/G>: Go to top/bottom",
                        Style::default(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  <f>: Scroll page forward,  <b>: Scroll page backward",
                        Style::default(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  <Enter>: Open bucket",
                        Style::default(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  <x>: Open management console in browser",
                        Style::default(),
                    )),
                ]
            }
            ViewState::ObjectList => {
                vec![
                    Line::from(Span::styled(
                        "  <Esc> <Ctrl-c>: Quit app,  <j/k>: Select item,  <g/G>: Go to top/bottom",
                        Style::default(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  <f>: Scroll page forward,  <b>: Scroll page backward",
                        Style::default(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  <Enter>: Open file or folder,  <Backspace>: Go back to prev folder",
                        Style::default(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  <~>: Go back to bucket list",
                        Style::default(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  <x>: Open management console in browser",
                        Style::default(),
                    )),
                ]
            }
            ViewState::Detail(_) => {
                vec![
                    Line::from(Span::styled(
                        "  <Esc> <Ctrl-c>: Quit app,  <h/l>: Select tabs,  <Backspace>: Close detail panel",
                        Style::default(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "  <s>: Download object,  <x>: Open management console in browser",
                        Style::default(),
                    )),
                ]
            }
        }
    .into_iter();

    let content: Vec<Line> = app_detail.chain(help).collect();
    Paragraph::new(content).block(Block::default().title(APP_NAME).borders(Borders::all()))
}

fn build_short_help(app: &App) -> Paragraph {
    let help = match app.app_view_state.view_state {
        ViewState::Initializing => "",
        ViewState::BucketList => "<Esc>: Quit, <j/k>: Select, <Enter>: Open, <?> Help",
        ViewState::ObjectList => {
            "<Esc>: Quit, <j/k>: Select, <Enter>: Open, <Backspace>: Go back, <?> Help"
        }
        ViewState::Detail(_) => {
            "<Esc>: Quit, <h/l>: Select tabs, <s>: Download, <Backspace>: Close, <?> Help"
        }
        ViewState::Help(_) => "<Esc>: Quit, <?>: Close help",
    };
    Paragraph::new(Span::styled(help, Style::default().fg(SHORT_HELP_COLOR)))
        .block(Block::default().padding(Padding::horizontal(2)))
}

fn build_info_status(msg: &String) -> Paragraph {
    Paragraph::new(Span::styled(
        msg,
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(INFO_STATUS_COLOR),
    ))
    .block(Block::default().padding(Padding::horizontal(2)))
}

fn build_error_status(err: &String) -> Paragraph {
    let err = format!("ERROR: {}", err);
    Paragraph::new(Span::styled(
        err,
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(ERROR_STATUS_COLOR),
    ))
    .block(Block::default().padding(Padding::horizontal(2)))
}

fn render_footer(f: &mut Frame, area: Rect, app: &App) {
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
        Line::from(""),
        Line::from(Span::styled(
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

fn calc_loading_dialog_rect(r: Rect) -> Rect {
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

fn render_loading_dialog(f: &mut Frame, app: &App) {
    if app.app_view_state.is_loading {
        let loading = build_loading_dialog("Loading...");
        let area = calc_loading_dialog_rect(f.size());
        f.render_widget(Clear, area);
        f.render_widget(loading, area);
    }
}
