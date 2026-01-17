use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("cairo error: {0}")]
    Cairo(#[from] cairo::Error),

    #[error("failed to create surface")]
    SurfaceCreationFailed,

    #[error("failed to create context")]
    ContextCreationFailed,

    #[error("X11 error: {0}")]
    X11(#[from] gartk_x11::X11Error),

    #[error("invalid font specification: {0}")]
    InvalidFont(String),

    #[error("pango error")]
    Pango,
}

pub type Result<T> = std::result::Result<T, RenderError>;
