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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub role: Role,
    pub text: String,
    pub bounds: VomRect,
    pub selected: bool,
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
