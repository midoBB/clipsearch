use crate::*;
use iced::widget::image;
use iced::widget::text_input::StyleSheet;
use iced::widget::{column, container, row, scrollable, text, text_input};
use iced::{
    alignment, event, executor, keyboard, Application, Color, Command, Element, Length, Settings,
    Subscription, Theme,
};
use img::guess_format;
use img::ImageFormat;
use redb::Database;
use std::io::Cursor;
use std::path::PathBuf;
use std::result::Result::Ok;
use std::sync::Arc;

pub struct ClipSearch {
    search_input: String,
    items: Vec<Arc<[u8]>>,
    selected_item: Option<usize>,
    db: Arc<Database>,
    socket_path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum Message {
    SearchInputChanged(String),
    ItemSelected(usize),
    ShortcutPressed(usize),
}

pub struct Flags {
    pub db: Arc<Database>,
    pub socket_path: PathBuf,
}

fn preview<'a, Message: 'a>(index: usize, data: &[u8]) -> Element<'a, Message> {
    // First, try to determine if it's an image
    if let Ok(format) = guess_format(data) {
        match format {
            ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Gif | ImageFormat::WebP => {
                if let Ok(_) = img::load_from_memory(data) {
                    let handle = iced::widget::image::Handle::from_memory(data.to_vec());
                    return container(image(handle))
                        .width(Length::Fill)
                        .height(Length::Shrink)
                        .into();
                }
            }
            _ => {} // If it's another image format, fall through to text preview
        }
    }

    // If it's not a recognized image, treat it as text
    let prev = String::from_utf8_lossy(data);
    let prev = prev.trim();
    let prev = prev.split_whitespace().collect::<Vec<&str>>().join(" ");
    let prev = trunc(&prev, 80, "…");

    container(text(format!("{}{}{}", index, FIELD_SEP, prev)))
        .width(Length::Fill)
        .height(Length::Shrink)
        .into()
}

// Helper functions remain the same
const FIELD_SEP: &str = "\t";

fn trunc(s: &str, max_width: usize, ellipsis: &str) -> String {
    if s.chars().count() <= max_width {
        s.to_string()
    } else {
        let mut result: String = s.chars().take(max_width - ellipsis.len()).collect();
        result.push_str(ellipsis);
        result
    }
}

impl Application for ClipSearch {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = Flags;

    fn new(flags: Flags) -> (Self, Command<Message>) {
        let items = load_last_items_from_db(&flags.db).unwrap_or_else(|_| vec![]);
        (
            ClipSearch {
                search_input: String::new(),
                items,
                selected_item: Some(0),
                db: flags.db,
                socket_path: flags.socket_path,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        env!("CARGO_PKG_NAME").to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SearchInputChanged(new_value) => {
                self.search_input = new_value;
                Command::none()
            }
            Message::ItemSelected(index) => {
                self.selected_item = Some(index);
                Command::none()
            }
            Message::ShortcutPressed(index) => {
                if index < self.items.len() {
                    self.selected_item = Some(index);
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let search_bar = text_input("Search...", &self.search_input)
            .on_input(Message::SearchInputChanged)
            .padding(10)
            .size(20)
            .width(Length::Fill);
        let items: Element<_> = self
            .items
            .iter()
            .enumerate()
            .fold(column(None).spacing(10), |col, (index, item)| {
                let preview_element = preview(index, item);
                col.push(preview_element)
            })
            .into();

        // Combine search bar and items in a column layout
        let content = column(None)
            .push(search_bar)
            .push(scrollable(items))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(alignment::Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn load_last_items_from_db(db: &Database) -> Result<Vec<Arc<[u8]>>, anyhow::Error> {
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(CLIPBOARD_TABLE)?;

    let res: Vec<Arc<[u8]>> = table
        .iter()?
        .rev()
        .take(10)
        .filter_map(|item| {
            if let Ok((_, value)) = item {
                // Create Arc<[u8]> from Vec<u8> slice
                Some(Arc::from(value.value().to_vec()))
            } else {
                None
            }
        })
        .collect(); // Collect Vec<Arc<[u8]>>

    Ok(res)
}
