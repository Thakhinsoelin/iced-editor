use std::sync::Arc;
use iced::{font, Font, executor, Application, Command, Element, Length, Settings, Theme};
use iced::widget::{button, column, container, horizontal_space, row, text, text_editor};

use std::io;
use std::path::{Path, PathBuf};
fn main() -> iced::Result {
    Editor::run(Settings {
        fonts: vec![include_bytes!("../fonts/font/editor-icons.ttf")
            .as_slice()
            .into()],
        ..Settings::default()
    })
}


struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>
}

#[derive(Debug, Clone)]
enum Messages {
    Edit(text_editor::Action),
    New,
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    Save,
    File_Saved(Result<PathBuf, Error>),
}

impl Application for Editor {
    type Executor = executor::Default;
    type Message = Messages;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Messages>) {
        (
            Self {
                path: None,
                content: text_editor::Content::new(),
                error: None,
            },
            Command::perform(
                load_file(default_file()), 
                Messages::FileOpened,
            ),
        )
    }

    fn title(&self) -> String {
        String::from("A cool editor!")
    }

    fn update(&mut self, message: Messages) -> Command<Messages>{
        match message {
            Messages::Edit(action) => {
                self.content.edit(action);
                self.error = None;
                Command::none()
            }
            Messages::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                Command::none()
            }
            Messages::Save => {
                let text = self.content.text();
                Command::perform(save_file(self.path.clone(), text), Messages::File_Saved)
            }
            Messages::File_Saved(Ok(Path)) => {
                self.path = Some(Path);
                Command::none()
            }
            Messages::File_Saved(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Messages::Open => {
                Command::perform(pick_file(), Messages::FileOpened)
            }
            Messages::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content); 
                Command::none()
            }

            Messages::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
        }
        
    
    }

    fn view(&self) -> Element<'_, Messages> {
        let controls = row![
            button("New").on_press(Messages::New),
            button("Open").on_press(Messages::Open),
            button("Save").on_press(Messages::Save),
        ].spacing(10);
        let input = text_editor(&self.content).on_edit(Messages::Edit);


        let status_bar = {
            let status = if let Some(Error::IO(error)) = self.error.as_ref() {
                text(error.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) => text(path).size(14),
                    None => text("New file"),
                }
            };

            let position = {
                let (line, column) = self.content.cursor_position();

                text(format!("{}:{}", line + 1, column + 1))
            };
            row![status, horizontal_space(Length::Fill), position]
        };
        container(column![controls, input, status_bar].spacing(10))
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(|error| Error::IO(error))?;

    Ok((path, contents))
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new().set_title("Choose a text file...").pick_file().await.ok_or(Error::DialogClosed)?;
    load_file(handle.path().to_owned()).await
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog::new()
            .set_title("Choose a file name...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    tokio::fs::write(&path, &text).await.map_err(|error| Error::IO(error.kind()))?;
    Ok(path)
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind)
}

fn icon<'a, Message>(codepoint: char) -> Element<'a, Message>{
    const ICON_FONT: font = Font::with_name("editor-icons");
    text(codepoint).font(ICON_FONT).into()
}

fn new_icon<'a, Message>() -> Element<'a, Messages> {
    todo!()
}
fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}