use crate::vom::{VomGrid, VomRect, VomStyle, Cluster, Component, Role, VomColor};

const BRAILLE_SPINNERS: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const CIRCLE_SPINNERS: [char; 4] = ['◐', '◑', '◒', '◓'];
const STATUS_CHARS: [char; 4] = ['✓', '✔', '✗', '✘'];
const ROUNDED_CORNERS: [char; 4] = ['╭', '╮', '╰', '╯'];
const BOX_CHARS: [char; 22] = [
    '─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼', '═', '║', '╔', '╗', '╚', '╝', '╠', '╣',
    '╦', '╩', '╬',
];
const MIN_BUTTON_LENGTH: usize = 3;
const PROGRESS_BAR_CHARS: [char; 8] = ['=', '>', '#', '.', '█', '▓', '░', '-'];
const MENU_ITEM_DASH_PREFIX: &str = "- ";
const PROGRESS_FILLED: [char; 4] = ['█', '▓', '▒', '='];
const PROGRESS_EMPTY: [char; 4] = ['░', '▒', ' ', '.'];
const PROGRESS_ARROW: char = '>';
const ERROR_PREFIXES: [&str; 6] = ["Error:", "error:", "ERROR:", "Error ", "error ", "ERROR "];
const FAILURE_CHARS: [char; 2] = ['✗', '✘'];
const CODE_BLOCK_BORDER: char = '│';

const TAB_BG_BLUE: u8 = 4;
const TAB_BG_CYAN: u8 = 6;

#[derive(Debug, Clone)]
pub struct ClassifyOptions {
    pub tab_row_threshold: usize,
}

impl Default for ClassifyOptions {
    fn default() -> Self {
        Self {
            tab_row_threshold: 2,
        }
    }
}

pub fn analyze(grid: &dyn VomGrid, cursor: (usize, usize)) -> Vec<Component> {
    let clusters = segment(grid);
    classify(clusters, cursor, &ClassifyOptions::default())
}

fn segment(grid: &dyn VomGrid) -> Vec<Cluster> {
    let (rows, cols) = grid.grid_dimensions();
    let mut clusters = Vec::new();

    for r in 0..rows {
        let mut current_text = String::new();
        let mut current_style: Option<VomStyle> = None;
        let mut start_col = 0;

        for c in 0..cols {
            if let Some((ch, style)) = grid.cell(r, c) {
                if current_style.is_none() {
                    current_text.push(ch);
                    current_style = Some(style);
                    start_col = c;
                } else if Some(style) == current_style {
                    current_text.push(ch);
                } else {
                    // Flush current cluster
                    if let Some(s) = current_style {
                        push_cluster(&mut clusters, &current_text, s, r, start_col);
                    }
                    current_text = String::from(ch);
                    current_style = Some(style);
                    start_col = c;
                }
            }
        }
        if !current_text.is_empty() {
            if let Some(s) = current_style {
                push_cluster(&mut clusters, &current_text, s, r, start_col);
            }
        }
    }
    clusters
}

fn push_cluster(clusters: &mut Vec<Cluster>, text: &str, style: VomStyle, row: usize, col: usize) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    // Find actual start and end of non-whitespace
    let start_offset = text.find(|c: char| !c.is_whitespace()).unwrap_or(0);
    let end_offset = text.rfind(|c: char| !c.is_whitespace()).unwrap_or(text.len() - 1);

    clusters.push(Cluster {
        text: trimmed.to_string(),
        style,
        bounds: VomRect {
            x: col + start_offset,
            y: row,
            width: end_offset - start_offset + 1,
            height: 1,
        },
    });
}

fn classify(clusters: Vec<Cluster>, cursor: (usize, usize), options: &ClassifyOptions) -> Vec<Component> {
    let mut components = Vec::new();
    let mut role_counts = std::collections::HashMap::new();

    for cluster in clusters {
        let text = cluster.text.trim();
        if text.is_empty() { continue; }

        let role = infer_role(&cluster, cursor, options);

        let entry = role_counts.entry(role).or_insert(0);
        *entry += 1;
        let count = *entry;

        let id_prefix = match role {
            Role::Button => "btn",
            Role::Input => "inp",
            Role::Checkbox => "chk",
            Role::Tab => "tab",
            Role::MenuItem => "menu",
            Role::Link => "link",
            Role::ProgressBar => "prog",
            Role::Status => "stat",
            Role::ErrorMessage => "err",
            Role::DiffLine => "diff",
            Role::CodeBlock => "code",
            Role::Panel => "pan",
            Role::ToolBlock => "tool",
            Role::PromptMarker => "prom",
            Role::StaticText => "txt",
            Role::Radio => "rad",
            Role::Select => "sel",
        };

        let id = format!("@{}{}", id_prefix, count);

        components.push(Component {
            id,
            role,
            text: text.to_string(),
            bounds: cluster.bounds,
            selected: is_selected(&cluster),
            checked: detect_checked_state(text),
            value: if role == Role::Input { Some(text.to_string()) } else { None },
        });
    }

    components
}

pub fn detect_checked_state(text: &str) -> Option<bool> {
    let lower = text.to_lowercase();
    if lower.contains("[x]")
        || lower.contains("(x)")
        || lower.contains("☑")
        || lower.contains("✓")
        || lower.contains("✔")
        || lower.contains("◉")
        || lower.contains("●")
    {
        Some(true)
    } else if lower.contains("[ ]")
        || lower.contains("( )")
        || lower.contains("☐")
        || lower.contains("◯")
        || lower.contains("○")
    {
        Some(false)
    } else {
        None
    }
}

fn is_selected(cluster: &Cluster) -> bool {
    cluster.style.inverse
        || cluster.text.starts_with('❯')
        || cluster.text.starts_with('›')
        || cluster.text.starts_with('◉')
        || (cluster.text.starts_with('>') && !cluster.text.starts_with(">>"))
}

fn infer_role(cluster: &Cluster, cursor: (usize, usize), options: &ClassifyOptions) -> Role {
    let text = cluster.text.trim();
    let (cursor_row, cursor_col) = cursor;

    // Cursor interaction
    if cluster.bounds.y == cursor_row &&
       cursor_col >= cluster.bounds.x &&
       cursor_col < cluster.bounds.x + cluster.bounds.width {
        return Role::Input;
    }

    if is_button_text(text) {
        return Role::Button;
    }

    if cluster.style.inverse {
        if cluster.bounds.y <= options.tab_row_threshold {
            return Role::Tab;
        }
        return Role::MenuItem;
    }

    if let VomColor::Indexed(idx) = cluster.style.bg {
        if idx == TAB_BG_BLUE || idx == TAB_BG_CYAN {
            return Role::Tab;
        }
    }

    if is_error_message(text) {
        return Role::ErrorMessage;
    }

    if is_input_field(text) {
        return Role::Input;
    }

    if is_checkbox(text) {
        return Role::Checkbox;
    }

    if is_radio(text) {
        return Role::Radio;
    }

    if is_select(text) {
        return Role::Select;
    }

    if is_prompt_marker(text) {
        return Role::PromptMarker;
    }

    if is_menu_item(text) {
        return Role::MenuItem;
    }

    if is_link(text) {
        return Role::Link;
    }

    if is_progress_bar(text) {
        return Role::ProgressBar;
    }

    if is_diff_line(text) {
        return Role::DiffLine;
    }

    if is_tool_block_border(text) {
        return Role::ToolBlock;
    }

    if is_code_block_border(text) {
        return Role::CodeBlock;
    }

    if is_panel_border(text) {
        return Role::Panel;
    }

    if is_status_indicator(text) {
        return Role::Status;
    }

    Role::StaticText
}

// Patterns ported from agent-tui

pub fn is_button_text(text: &str) -> bool {
    if text.len() < MIN_BUTTON_LENGTH {
        return false;
    }

    if let Some(inner) = text.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let trimmed = inner.trim();
        if matches!(trimmed, "x" | "X" | " " | "" | "✓" | "✔") {
            return false;
        }

        if inner.chars().any(|c| c.is_alphabetic()) {
            return true;
        }

        let (progress_chars, non_space_chars) = inner.chars().fold((0, 0), |(p, n), c| {
            (
                if PROGRESS_BAR_CHARS.contains(&c) {
                    p + 1
                } else {
                    p
                },
                if !c.is_whitespace() { n + 1 } else { n },
            )
        });
        if non_space_chars > 0 && progress_chars > non_space_chars / 2 {
            return false;
        }
        return true;
    }

    if let Some(inner) = text.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        let trimmed = inner.trim();
        return !matches!(trimmed, "" | " " | "o" | "O" | "●" | "◉");
    }

    text.starts_with('<') && text.ends_with('>')
}

pub fn is_input_field(text: &str) -> bool {
    if text.contains("___") {
        return true;
    }

    if !text.is_empty() && text.chars().all(|ch| ch == '_') {
        return true;
    }

    if text.ends_with(": _") || text.ends_with(":_") {
        return true;
    }

    false
}

pub fn is_checkbox(text: &str) -> bool {
    matches!(
        text,
        "[x]"
            | "[X]"
            | "[ ]"
            | "[✓]"
            | "[✔]"
            | "◼"
            | "◻"
            | "☐"
            | "☑"
            | "☒"
    )
}

pub fn is_radio(text: &str) -> bool {
    matches!(
        text,
        "(x)" | "(X)" | "( )" | "◉" | "◯" | "●" | "○"
    )
}

pub fn is_select(text: &str) -> bool {
    text.starts_with('❯') || text.starts_with('›')
}

pub fn is_menu_item(text: &str) -> bool {
    text.starts_with('>')
        || text.starts_with('❯')
        || text.starts_with('›')
        || text.starts_with('→')
        || text.starts_with('▶')
        || text.starts_with("• ")
        || text.starts_with("* ")
        || text.starts_with(MENU_ITEM_DASH_PREFIX)
}

pub fn is_panel_border(text: &str) -> bool {
    let total = text.chars().filter(|c| !c.is_whitespace()).count();
    if total == 0 {
        return false;
    }

    let box_count = text.chars().filter(|c| BOX_CHARS.contains(c)).count();
    box_count > total / 2
}

pub fn is_status_indicator(text: &str) -> bool {
    let text = text.trim();
    let Some(first_char) = text.chars().next() else {
        return false;
    };

    BRAILLE_SPINNERS.contains(&first_char)
        || CIRCLE_SPINNERS.contains(&first_char)
        || STATUS_CHARS.contains(&first_char)
}

pub fn is_tool_block_border(text: &str) -> bool {
    let text = text.trim();
    let Some(first_char) = text.chars().next() else {
        return false;
    };

    let last_char = text
        .chars()
        .last()
        .expect("non-empty string has a last char");

    ROUNDED_CORNERS.contains(&first_char) || ROUNDED_CORNERS.contains(&last_char)
}

pub fn is_prompt_marker(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed == ">" || trimmed == "> "
}

pub fn is_progress_bar(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    if text.starts_with('[') && text.ends_with(']') {
        let inner = &text[1..text.len() - 1];
        if inner.is_empty() {
            return false;
        }
        let progress_chars: usize = inner
            .chars()
            .filter(|c| PROGRESS_FILLED.contains(c) || *c == PROGRESS_ARROW || *c == '#')
            .count();
        let empty_chars: usize = inner
            .chars()
            .filter(|c| PROGRESS_EMPTY.contains(c) || *c == '-')
            .count();
        return progress_chars + empty_chars > inner.len() / 2;
    }

    let total_chars = text.chars().count();
    let progress_chars: usize = text
        .chars()
        .filter(|c| PROGRESS_FILLED.contains(c) || PROGRESS_EMPTY.contains(c))
        .count();

    progress_chars > total_chars / 2
}

pub fn is_link(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    if text.starts_with("https://")
        || text.starts_with("http://")
        || text.starts_with("file://")
        || text.starts_with("ftp://")
    {
        return true;
    }

    is_file_path(text)
}

fn is_file_path(text: &str) -> bool {
    let path_part = text.split(':').next().unwrap_or(text);

    if path_part.starts_with('/') && path_part.len() > 1 {
        return has_file_extension(path_part) || path_part.contains('/');
    }

    if path_part.starts_with("./") || path_part.starts_with("../") {
        return true;
    }

    if path_part.contains('/') && has_file_extension(path_part) {
        return true;
    }

    false
}

fn has_file_extension(text: &str) -> bool {
    const EXTENSIONS: [&str; 30] = [
        ".rs", ".js", ".ts", ".tsx", ".jsx", ".py", ".go", ".java", ".c", ".cpp", ".h", ".hpp",
        ".md", ".txt", ".json", ".yaml", ".yml", ".toml", ".html", ".css", ".sh", ".sql", ".xml",
        ".vue", ".svelte", ".rb", ".php", ".swift", ".kt", ".scala",
    ];
    EXTENSIONS.iter().any(|ext| text.ends_with(ext))
}

pub fn is_error_message(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    if ERROR_PREFIXES.iter().any(|prefix| text.starts_with(prefix)) {
        return true;
    }

    if let Some(first_char) = text.chars().next() {
        if FAILURE_CHARS.contains(&first_char) {
            return true;
        }
    }

    false
}

pub fn is_diff_line(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    if text.starts_with("@@") {
        return true;
    }

    if text.starts_with('+') && text.len() > 1 {
        return true;
    }

    if text.starts_with('-') && text.len() > 1 {
        return true;
    }

    false
}

pub fn is_code_block_border(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    if !text.contains(CODE_BLOCK_BORDER) {
        return false;
    }

    const CORNER_CHARS: [char; 8] = ['┌', '┐', '└', '┘', '╭', '╮', '╰', '╯'];
    if text.chars().any(|c| CORNER_CHARS.contains(&c)) {
        return false;
    }

    let border_count = text.chars().filter(|c| *c == CODE_BLOCK_BORDER).count();

    (1..=3).contains(&border_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vom::{VomGrid, VomStyle};

    struct MockGrid {
        cells: Vec<Vec<(char, VomStyle)>>,
    }

    impl VomGrid for MockGrid {
        fn grid_dimensions(&self) -> (usize, usize) {
            (self.cells.len(), self.cells[0].len())
        }
        fn cell(&self, row: usize, col: usize) -> Option<(char, VomStyle)> {
            self.cells.get(row).and_then(|r| r.get(col)).cloned()
        }
    }

    #[test]
    fn test_segmentation() {
        let mut grid = MockGrid {
            cells: vec![vec![(' ', VomStyle::default()); 10]; 3],
        };
        grid.cells[1][2] = ('[', VomStyle::default());
        grid.cells[1][3] = ('O', VomStyle::default());
        grid.cells[1][4] = ('K', VomStyle::default());
        grid.cells[1][5] = (']', VomStyle::default());

        let components = analyze(&grid, (0, 0));
        let btn = components.iter().find(|c| c.role == Role::Button).unwrap();
        assert_eq!(btn.text, "[OK]");
    }

    #[test]
    fn test_progress_bar_detection() {
        let mut grid = MockGrid {
            cells: vec![vec![(' ', VomStyle::default()); 20]; 1],
        };
        let text = "[|||||    ]";
        for (i, c) in text.chars().enumerate() {
            grid.cells[0][i] = (c, VomStyle::default());
        }

        // agent-tui uses '█', '▓', '▒', '=', '>', '#' for filled
        // and '░', '▒', ' ', '.', '-' for empty
        // '|' is NOT in PROGRESS_FILLED in agent-tui patterns.rs:
        // const PROGRESS_FILLED: [char; 4] = ['█', '▓', '▒', '='];
        // const PROGRESS_ARROW: char = '>';
        // But in is_progress_bar: .filter(|c| PROGRESS_FILLED.contains(c) || *c == PROGRESS_ARROW || *c == '#')
        // So '|' is NOT there.
        // Wait, looking at my previous read of patterns.rs:
        // const PROGRESS_FILLED: [char; 4] = ['█', '▓', '▒', '='];
        // It does not include '|'.

        // So the previous test case `[|||||    ]` might fail if I use agent-tui logic strictly.
        // Let's check agent-tui's tests.
        // test_progress_bar_block_style uses "████░░░░"
        // test_progress_bar_bracket_style uses "[===>    ]"

        // So I should update this test case to use compatible characters.

        let text_compatible = "[====>    ]";
        for (i, c) in text_compatible.chars().enumerate() {
            grid.cells[0][i] = (c, VomStyle::default());
        }

        let components = analyze(&grid, (99, 99));
        assert_eq!(components[0].role, Role::ProgressBar);
    }

    #[test]
    fn test_patterns_match_agent_tui() {
        assert!(is_button_text("[OK]"));
        assert!(!is_button_text("[x]")); // Checkbox
        assert!(is_input_field("Name: ___"));
        assert!(is_checkbox("[x]"));
        assert!(is_radio("(x)"));
        assert!(is_radio("◉"));
        assert!(is_select("❯ Option"));
        assert!(is_menu_item("> Option"));
        assert!(is_link("https://google.com"));
        assert!(is_error_message("Error: failed"));
        assert!(is_diff_line("+ added"));
    }
}
