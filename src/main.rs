use std::sync::Arc;
use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use iced::widget::{button, column, container, horizontal_space, row, text, text_editor};

use std::io;
use std::path::{Path, PathBuf};
fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>
}

#[derive(Debug, Clone)]
enum Messages {
    Edit(text_editor::Action),
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>)
}

impl Application for Editor {
    type Message = Messages;
    type Theme = Theme;
    type Executor = executor::Default;
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
        let controls = row![button("Open").on_press(Messages::Open)];
        let input = text_editor(&self.content).on_edit(Messages::Edit);

        let file_path = match self.path.as_deref().and_then(Path::to_str){
            Some(path) => text(path).size(14),
            
            None => text(""),
        };

        let position = {
            let (line, column) = self.content.cursor_position();

            text(format!("{}:{}", line + 1, column + 1))
        };
        let status_bar = row![file_path, horizontal_space(Length::Fill), position];
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

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind)
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}