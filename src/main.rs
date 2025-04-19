use std::sync::Arc;
use iced::{font, Font, executor, Application, Command, Element, Length, Settings, Theme, Subscription};
use iced::widget::{pick_list, button, column, container, horizontal_space, row, text, text_editor, tooltip};
use iced::theme;
use iced::keyboard;
use iced::highlighter::{self, Highlighter};
use std::io;
use std::path::{Path, PathBuf};

//Lets see if it works

fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: Font::MONOSPACE,
        fonts: vec![include_bytes!("../fonts/font/editor-icons.ttf")
            .as_slice()
            .into()],
        ..Settings::default()
    })
}


struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
    theme: highlighter::Theme,
    is_dirty: bool,
}

#[derive(Debug, Clone)]
enum Messages {
    Edit(text_editor::Action),
    New,
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    Save,
    File_Saved(Result<PathBuf, Error>),
    Theme_Selected(highlighter::Theme),
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
                theme: highlighter::Theme::SolarizedDark,
                is_dirty: true,
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
                self.is_dirty = self.is_dirty || action.is_edit();
                self.error = None;
                self.content.edit(action);
                Command::none()
            }
            Messages::New => {
                self.is_dirty = true;
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
                self.is_dirty = false;

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
                self.is_dirty = false;
                self.path = Some(path);
                self.content = text_editor::Content::with(&content); 
                Command::none()
            }

            Messages::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Messages::Theme_Selected(theme) => {
                self.theme = theme;
                Command::none()
            }
        }
        
    
    }

    fn subscription(&self) -> Subscription<Messages> {
        keyboard::on_key_press(|key_code, modifiers|{
            match key_code {
                keyboard::KeyCode::S if modifiers.command() => {
                    Some(Messages::Save)
                },
                _ => None,
            }
        })
    }

    fn view(&self) -> Element<'_, Messages> {
        let controls = row![
            action(new_icon(), "Create a New File", Some(Messages::New)),
            action(save_icon(), "Save File", self.is_dirty.then_some(Messages::Save)),
            action(open_icon(),"Open File", Some(Messages::Open)),
            horizontal_space(Length::Fill),
            pick_list(highlighter::Theme::ALL, Some(self.theme), Messages::Theme_Selected)
        ].spacing(10);
        let input = text_editor(&self.content)
            .on_edit(Messages::Edit)
            .highlight::<Highlighter>(highlighter::Settings {
                theme: self.theme,
                extension: self
                    .path
                    .as_ref()
                    .and_then(|path| path.extension()?.to_str())
                    .unwrap_or("rs").to_string(),
            }, |highlight, _theme| {
                highlight.to_format()
            });


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
        if self.theme.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
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


fn new_icon<'a>() -> Element<'a, Messages> {
    icon('\u{E800}')
}

fn open_icon<'a>() -> Element<'a, Messages> {
    icon('\u{E802}')
}

fn save_icon<'a>() -> Element<'a, Messages> {
    icon('\u{E801}')
}
fn icon<'a, Message>(codepoint: char) -> Element<'a, Message>{
    const ICON_FONT: Font = Font::with_name("editor-icons");
    text(codepoint).font(ICON_FONT).into()
}

fn action<'a>(content: Element<'a, Messages>, label: &str, on_press: Option<Messages>) -> Element<'a, Messages> {
    let is_disabled = on_press.is_none();
    tooltip(
        button(container(content).width(30).center_x())
            .on_press_maybe(on_press)
            .padding([5, 10]).style(
            if is_disabled {
                theme::Button::Secondary
            } else {
                theme::Button::Primary
            }
        ),
        label,
        tooltip::Position::FollowCursor
    )
        .style(theme::Container::Box)
        .into()
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}
