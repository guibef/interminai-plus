use crate::vom::{VomGrid, VomRect, VomStyle, Cluster, Component, Role, VomColor};

pub fn analyze(grid: &dyn VomGrid, cursor: (usize, usize)) -> Vec<Component> {
    let clusters = segment(grid);
    classify(clusters, cursor)
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

fn classify(clusters: Vec<Cluster>, cursor: (usize, usize)) -> Vec<Component> {
    let mut components = Vec::new();
    let mut role_counts = std::collections::HashMap::new();

    for cluster in clusters {
        let text = cluster.text.trim();
        if text.is_empty() { continue; }

        let role = infer_role(&cluster, cursor);

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
        };

        let id = format!("@{}{}", id_prefix, count);

        components.push(Component {
            id,
            role,
            text: text.to_string(),
            bounds: cluster.bounds,
            selected: is_selected(&cluster),
        });
    }

    components
}

fn is_selected(cluster: &Cluster) -> bool {
    cluster.style.inverse || cluster.text.starts_with('❯')
}

fn infer_role(cluster: &Cluster, cursor: (usize, usize)) -> Role {
    let text = cluster.text.trim();
    let (cursor_row, cursor_col) = cursor;

    // 1. Highest priority: explicit cursor interaction
    if cluster.bounds.y == cursor_row &&
       cursor_col >= cluster.bounds.x &&
       cursor_col < cluster.bounds.x + cluster.bounds.width {
        return Role::Input;
    }

    // 2. Pattern based detection (checked BEFORE style markers to catch things like [OK] even if not inversed)
    if is_button_text(text) {
        return Role::Button;
    }

    // 3. Structural/Style markers
    if cluster.style.inverse {
        if cluster.bounds.y <= 2 {
            return Role::Tab;
        }
        // In htop, the whole line is inversed when selected.
        // Agent-tui seems to classify these as buttons.
        return Role::Button;
    }

    if let VomColor::Indexed(idx) = cluster.style.bg {
        if idx == 4 || idx == 6 { // Blue/Cyan often used for tabs
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

fn is_button_text(text: &str) -> bool {
    if text.len() < 2 {
        return false;
    }

    if (text.starts_with('[') && text.ends_with(']')) ||
       (text.starts_with('(') && text.ends_with(')')) ||
       (text.starts_with('<') && text.ends_with('>')) {
        let inner = &text[1..text.len()-1];
        let trimmed = inner.trim();

        // Radio button / Checkbox exclusion
        if trimmed.is_empty() || matches!(trimmed, "x" | "X" | " " | "✓" | "✔" | "o" | "O" | "●" | "○" | "◉") {
            return false;
        }

        // If it contains alphabetic characters, it's likely a button
        if inner.chars().any(|c| c.is_alphabetic()) {
            return true;
        }

        // Exclude progress bars
        const PROGRESS_CHARS: [char; 5] = ['|', '=', '#', '>', '.'];
        let (progress_chars, non_space_chars) = inner.chars().fold((0, 0), |(p, n), c| {
            (
                if PROGRESS_CHARS.contains(&c) { p + 1 } else { p },
                if !c.is_whitespace() { n + 1 } else { n }
            )
        });
        if non_space_chars > 0 && progress_chars > non_space_chars / 2 {
            return false;
        }

        return true;
    }

    // htop style: F1Help (starts with F-key)
    if text.starts_with('F') && text.chars().nth(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
        return true;
    }

    // htop style: "Help", "Setup" etc. if they are single words and look like labels
    const COMMON_LABELS: [&str; 15] = [
        "Help", "Setup", "Search", "Filter", "List", "SortBy", "Nice", "Kill", "Quit",
        "OK", "Cancel", "Yes", "No", "Save", "Close"
    ];
    if COMMON_LABELS.contains(&text) {
        return true;
    }

    false
}

fn is_checkbox(text: &str) -> bool {
    matches!(
        text,
        "[x]" | "[X]" | "[ ]" | "[✓]" | "[✔]" | "◉" | "◯" | "●" | "○" | "◼" | "◻" | "☐" | "☑" | "☒"
    )
}

fn is_input_field(text: &str) -> bool {
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

fn is_menu_item(text: &str) -> bool {
    text.starts_with('>')
        || text.starts_with('❯')
        || text.starts_with('›')
        || text.starts_with('→')
        || text.starts_with('▶')
        || text.starts_with("• ")
        || text.starts_with("* ")
        || text.starts_with("- ")
}

fn is_link(text: &str) -> bool {
    let t = text.trim();
    if t.is_empty() { return false; }

    if t.starts_with("https://")
        || t.starts_with("http://")
        || t.starts_with("file://")
        || t.starts_with("ftp://")
    {
        return true;
    }

    is_file_path(t)
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

fn is_progress_bar(text: &str) -> bool {
    if text.is_empty() { return false; }

    if text.starts_with('[') && text.ends_with(']') {
        let inner = &text[1..text.len()-1];
        let bar_chars = inner.chars().filter(|&c| c == '|' || c == '=' || c == '#' || c == '>' || c == '.').count();
        return bar_chars > inner.len() / 2 && inner.len() > 2;
    }

    let block_chars = text.chars().filter(|&c| c == '█' || c == '▓' || c == '▒' || c == '░').count();
    block_chars > text.len() / 2 && text.len() > 2
}

fn is_status_indicator(text: &str) -> bool {
    let first = text.chars().next().unwrap_or(' ');
    matches!(first, '⠋'|'⠙'|'⠹'|'⠸'|'⠼'|'⠴'|'⠦'|'⠧'|'⠇'|'⠏' | '✓'|'✔'|'✗'|'✘' | '◐'|'◑'|'◒'|'◓')
}

fn is_error_message(text: &str) -> bool {
    let t = text.to_lowercase();
    t.starts_with("error:") || t.starts_with("failure:") || text.starts_with('✗') || text.starts_with('✘')
}

fn is_diff_line(text: &str) -> bool {
    text.starts_with("@@") || (text.starts_with('+') && text.len() > 1) || (text.starts_with('-') && text.len() > 1)
}

fn is_prompt_marker(text: &str) -> bool {
    text == ">" || text == "> "
}

fn is_tool_block_border(text: &str) -> bool {
    let first = text.chars().next().unwrap_or(' ');
    let last = text.chars().last().unwrap_or(' ');
    matches!(first, '╭'|'╰') || matches!(last, '╮'|'╯')
}

fn is_code_block_border(text: &str) -> bool {
    text.trim().starts_with('│') && !text.contains('┌') && !text.contains('└')
}

fn is_panel_border(text: &str) -> bool {
    const BOX_CHARS: [char; 22] = [
        '─', '│', '┌', '┐', '└', '┘', '├', '┤', '┬', '┴', '┼', '═', '║', '╔', '╗', '╚', '╝', '╠', '╣',
        '╦', '╩', '╬',
    ];
    let total = text.chars().filter(|c| !c.is_whitespace()).count();
    if total == 0 {
        return false;
    }

    let box_count = text.chars().filter(|c| BOX_CHARS.contains(c)).count();
    box_count > total / 2
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
    fn test_htop_fkey_button() {
        let mut grid = MockGrid {
            cells: vec![vec![(' ', VomStyle::default()); 20]; 1],
        };
        let text = "F1Help";
        for (i, c) in text.chars().enumerate() {
            grid.cells[0][i] = (c, VomStyle::default());
        }

        let components = analyze(&grid, (99, 99));
        assert_eq!(components[0].role, Role::Button);
        assert_eq!(components[0].text, "F1Help");
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

        let components = analyze(&grid, (99, 99));
        assert_eq!(components[0].role, Role::ProgressBar);
    }
}
