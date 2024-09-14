use iced::widget::text_input::StyleSheet;
use iced::widget::{column, container, row, scrollable, text, text_input};
use iced::{
    alignment, event, executor, keyboard, Application, Color, Command, Element, Length, Settings,
    Subscription, Theme,
};
use redb::Database;
use std::path::PathBuf;
use std::sync::Arc;

use crate::*;

pub struct ClipSearch {
    search_input: String,
    items: Vec<String>,
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
                    println!("Shortcut pressed for item: {}", self.items[index]);
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

            let item_container = container(
                text(item)
                    .size(20)
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Left)
            )
            .width(Length::Fill)
            .height(Length::Shrink);

            col.push(item_container)
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



fn load_last_items_from_db(db: &Database) -> Result<Vec<String>, anyhow::Error> {
    let read_txn = db.begin_read()?;
    let table = read_txn.open_table(CLIPBOARD_TABLE)?;
    let mut items = Vec::new();
    for (i, result) in table.iter()?.enumerate() {
        if i > 10 {
            break;
        }
        let (_, value) = result?;
        items.push(String::from_utf8_lossy(value.value()).to_string());
    }
    Ok(items)
}
