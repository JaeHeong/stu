use std::{
    fmt::{self, Debug, Formatter},
    sync::mpsc,
    thread,
};

use ratatui::crossterm::event::KeyEvent;

use crate::{
    client::Client,
    error::{AppError, Result},
    object::{BucketItem, FileDetail, FileVersion, ObjectItem, ObjectKey, RawObject},
};

#[derive(Debug)]
pub enum AppEventType {
    Key(KeyEvent),
    Resize(usize, usize),
    Initialize(Client, Option<String>),
    CompleteInitialize(Result<CompleteInitializeResult>),
    LoadObjects,
    CompleteLoadObjects(Result<CompleteLoadObjectsResult>),
    LoadObjectDetail,
    CompleteLoadObjectDetail(Result<CompleteLoadObjectDetailResult>),
    DownloadObject(FileDetail, Option<String>),
    DownloadObjectAs(FileDetail, String, Option<String>),
    CompleteDownloadObject(Result<CompleteDownloadObjectResult>),
    PreviewObject(FileDetail, Option<String>),
    CompletePreviewObject(Result<CompletePreviewObjectResult>),
    BucketListMoveDown,
    ObjectListMoveDown,
    ObjectListMoveUp,
    BackToBucketList,
    OpenPreview(FileDetail, Option<String>),
    DetailDownloadObject(FileDetail, Option<String>),
    DetailDownloadObjectAs(FileDetail, String, Option<String>),
    PreviewDownloadObject(RawObject, String),
    PreviewDownloadObjectAs(FileDetail, String, Option<String>),
    BucketListOpenManagementConsole,
    ObjectListOpenManagementConsole,
    ObjectDetailOpenManagementConsole(String),
    CloseCurrentPage,
    OpenHelp,
    CopyToClipboard(String, String),
    NotifyInfo(String),
    NotifySuccess(String),
    NotifyWarn(String),
    NotifyError(AppError),
    Quit,
}

#[derive(Debug)]
pub struct CompleteInitializeResult {
    pub buckets: Vec<BucketItem>,
}

impl CompleteInitializeResult {
    pub fn new(buckets: Result<Vec<BucketItem>>) -> Result<CompleteInitializeResult> {
        let buckets = buckets?;
        Ok(CompleteInitializeResult { buckets })
    }
}

#[derive(Debug)]
pub struct CompleteLoadObjectsResult {
    pub items: Vec<ObjectItem>,
}

impl CompleteLoadObjectsResult {
    pub fn new(items: Result<Vec<ObjectItem>>) -> Result<CompleteLoadObjectsResult> {
        let items = items?;
        Ok(CompleteLoadObjectsResult { items })
    }
}

#[derive(Debug)]
pub struct CompleteLoadObjectDetailResult {
    pub detail: Box<FileDetail>, // to avoid "warning: large size difference between variants" for AppEventType
    pub map_key: ObjectKey,
}

impl CompleteLoadObjectDetailResult {
    pub fn new(
        detail: Result<FileDetail>,
        map_key: ObjectKey,
    ) -> Result<CompleteLoadObjectDetailResult> {
        let detail = Box::new(detail?);
        Ok(CompleteLoadObjectDetailResult { detail, map_key })
    }
}

#[derive(Debug)]
pub struct CompleteDownloadObjectResult {
    pub obj: RawObject,
    pub path: String,
}

impl CompleteDownloadObjectResult {
    pub fn new(obj: Result<RawObject>, path: String) -> Result<CompleteDownloadObjectResult> {
        let obj = obj?;
        Ok(CompleteDownloadObjectResult { obj, path })
    }
}

#[derive(Debug)]
pub struct CompletePreviewObjectResult {
    pub obj: RawObject,
    pub file_detail: FileDetail,
    pub file_version_id: Option<String>,
    pub path: String,
}

impl CompletePreviewObjectResult {
    pub fn new(
        obj: Result<RawObject>,
        file_detail: FileDetail,
        file_version_id: Option<String>,
        path: String,
    ) -> Result<CompletePreviewObjectResult> {
        let obj = obj?;
        Ok(CompletePreviewObjectResult {
            obj,
            file_detail,
            file_version_id,
            path,
        })
    }
}

#[derive(Clone)]
pub struct Sender {
    tx: mpsc::Sender<AppEventType>,
}

impl Debug for Sender {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Sender")
    }
}

impl Sender {
    pub fn send(&self, event: AppEventType) {
        self.tx.send(event).unwrap();
    }
}

pub struct Receiver {
    rx: mpsc::Receiver<AppEventType>,
}

impl Receiver {
    pub fn recv(&self) -> AppEventType {
        self.rx.recv().unwrap()
    }
}

pub fn new() -> (Sender, Receiver) {
    let (tx, rx) = mpsc::channel();
    let tx = Sender { tx };
    let rx = Receiver { rx };

    let event_tx = tx.clone();
    thread::spawn(move || loop {
        match ratatui::crossterm::event::read() {
            Ok(e) => match e {
                ratatui::crossterm::event::Event::Key(key) => {
                    event_tx.send(AppEventType::Key(key));
                }
                ratatui::crossterm::event::Event::Resize(w, h) => {
                    event_tx.send(AppEventType::Resize(w as usize, h as usize));
                }
                _ => {}
            },
            Err(e) => {
                let e = AppError::new("Failed to read event", e);
                event_tx.send(AppEventType::NotifyError(e));
            }
        }
    });

    (tx, rx)
}
