use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VomColor {
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VomStyle {
    pub bold: bool,
    pub underline: bool,
    pub inverse: bool,
    pub fg: VomColor,
    pub bg: VomColor,
}

impl Default for VomStyle {
    fn default() -> Self {
        Self {
            bold: false,
            underline: false,
            inverse: false,
            fg: VomColor::Default,
            bg: VomColor::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VomRect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Button,
    Tab,
    Input,
    StaticText,
    Panel,
    Checkbox,
    MenuItem,
    Status,
    ToolBlock,
    PromptMarker,
    ProgressBar,
    Link,
    ErrorMessage,
    DiffLine,
    CodeBlock,
    Radio,
    Select,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Button => write!(f, "button"),
            Role::Tab => write!(f, "tab"),
            Role::Input => write!(f, "input"),
            Role::StaticText => write!(f, "text"),
            Role::Panel => write!(f, "panel"),
            Role::Checkbox => write!(f, "checkbox"),
            Role::MenuItem => write!(f, "menuitem"),
            Role::Status => write!(f, "status"),
            Role::ToolBlock => write!(f, "toolblock"),
            Role::PromptMarker => write!(f, "prompt"),
            Role::ProgressBar => write!(f, "progressbar"),
            Role::Link => write!(f, "link"),
            Role::ErrorMessage => write!(f, "error"),
            Role::DiffLine => write!(f, "diff"),
            Role::CodeBlock => write!(f, "codeblock"),
            Role::Radio => write!(f, "radio"),
            Role::Select => write!(f, "select"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub role: Role,
    pub text: String,
    pub bounds: VomRect,
    pub selected: bool,
    pub checked: Option<bool>,
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub text: String,
    pub style: VomStyle,
    pub bounds: VomRect,
}

pub trait VomGrid {
    fn grid_dimensions(&self) -> (usize, usize);
    fn cell(&self, row: usize, col: usize) -> Option<(char, VomStyle)>;
}

pub mod engine;
