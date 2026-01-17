use thiserror::Error;

#[derive(Error, Debug)]
pub enum X11Error {
    #[error("failed to connect to X11 display")]
    ConnectionFailed(#[from] x11rb::errors::ConnectError),

    #[error("X11 connection error: {0}")]
    Connection(#[from] x11rb::errors::ConnectionError),

    #[error("X11 reply error: {0}")]
    Reply(#[from] x11rb::errors::ReplyError),

    #[error("X11 reply or ID error: {0}")]
    ReplyOrId(#[from] x11rb::errors::ReplyOrIdError),

    #[error("no screens available")]
    NoScreens,

    #[error("invalid screen number: {0}")]
    InvalidScreen(usize),

    #[error("failed to create window")]
    WindowCreationFailed,

    #[error("failed to create colormap")]
    ColormapCreationFailed,

    #[error("no visual found with depth {0}")]
    NoVisual(u8),

    #[error("failed to grab keyboard")]
    KeyboardGrabFailed,

    #[error("failed to grab pointer")]
    PointerGrabFailed,

    #[error("RandR extension not available")]
    RandRNotAvailable,

    #[error("atom not found: {0}")]
    AtomNotFound(String),
}

pub type Result<T> = std::result::Result<T, X11Error>;
