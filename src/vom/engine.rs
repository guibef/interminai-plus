use crate::vom::{VomGrid, VomRect, VomStyle, Cluster, Component, Role};

pub fn analyze(grid: &dyn VomGrid, cursor: (usize, usize)) -> Vec<Component> {
    let clusters = segment(grid);
    classify(clusters, cursor)
}

fn segment(grid: &dyn VomGrid) -> Vec<Cluster> {
    let (rows, cols) = grid.grid_dimensions();
    let mut clusters = Vec::new();

    for r in 0..rows {
        let mut current_text = String::new();
        let mut current_style = VomStyle::default();
        let mut start_col = 0;

        for c in 0..cols {
            if let Some((ch, style)) = grid.cell(r, c) {
                if c == 0 {
                    current_text.push(ch);
                    current_style = style;
                    start_col = 0;
                } else if style == current_style {
                    current_text.push(ch);
                } else {
                    // Flush current cluster
                    push_cluster(&mut clusters, &current_text, current_style, r, start_col);
                    current_text = String::from(ch);
                    current_style = style;
                    start_col = c;
                }
            }
        }
        if !current_text.is_empty() {
            push_cluster(&mut clusters, &current_text, current_style, r, start_col);
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

    let cluster = Cluster {
        text: trimmed.to_string(),
        style,
        bounds: VomRect {
            x: col + start_offset,
            y: row,
            width: end_offset - start_offset + 1,
            height: 1,
        },
    };
    // println!("Pushing cluster: {:?}", cluster);
    clusters.push(cluster);
}

fn classify(clusters: Vec<Cluster>, cursor: (usize, usize)) -> Vec<Component> {
    let mut components = Vec::new();
    let mut btn_count = 0;
    let mut inp_count = 0;
    let mut txt_count = 0;
    let mut chk_count = 0;

    for cluster in clusters {
        let text = cluster.text.trim();
        if text.is_empty() { continue; }

        let (role, id_prefix, count) = if is_button(text, cluster.style) {
            btn_count += 1;
            (Role::Button, "btn", btn_count)
        } else if is_input(&cluster, cursor) {
            inp_count += 1;
            (Role::Input, "inp", inp_count)
        } else if is_checkbox(text) {
            chk_count += 1;
            (Role::Checkbox, "chk", chk_count)
        } else {
            txt_count += 1;
            (Role::StaticText, "txt", txt_count)
        };

        let id = format!("@{}{}", id_prefix, count);

        components.push(Component {
            id,
            role,
            text: text.to_string(),
            bounds: cluster.bounds,
            selected: cluster.style.inverse,
        });
    }

    components
}

fn is_button(text: &str, style: VomStyle) -> bool {
    // Buttons often have [ ] or < > or are inversed
    (text.starts_with('[') && text.ends_with(']')) ||
    (text.starts_with('<') && text.ends_with('>')) ||
    (text.starts_with('(') && text.ends_with(')')) ||
    style.inverse
}

fn is_checkbox(text: &str) -> bool {
    text.starts_with("[ ]") || text.starts_with("[x]") || text.starts_with("[X]")
}

fn is_input(cluster: &Cluster, cursor: (usize, usize)) -> bool {
    // Input is often where the cursor is, or a long underscore/blank with specific style
    let (cursor_row, cursor_col) = cursor;
    if cluster.bounds.y == cursor_row &&
       cursor_col >= cluster.bounds.x &&
       cursor_col < cluster.bounds.x + cluster.bounds.width {
        // If it looks like a label + input, it's safer to call it text unless it's ONLY underscores/spaces
        if cluster.text.chars().any(|c| c.is_alphanumeric()) {
             // Heuristic: if it ends with underscores and has the cursor, maybe it is an input
             // but for now let's be strict to avoid merging labels into inputs
             return cluster.text.chars().all(|c| c == '_' || c == ' ' || c == ':');
        }
        return true;
    }
    cluster.text.chars().all(|c| c == '_' || c == ' ') && cluster.bounds.width > 2
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
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].text, "[OK]");
        assert_eq!(components[0].role, Role::Button);
        assert_eq!(components[0].bounds.x, 2);
        assert_eq!(components[0].bounds.width, 4);
    }

    #[test]
    fn test_input_detection() {
        let mut grid = MockGrid {
            cells: vec![vec![(' ', VomStyle::default()); 15]; 3],
        };
        // Label: "Name:" at (1,1) to (1,5)
        grid.cells[1][1] = ('N', VomStyle::default());
        grid.cells[1][2] = ('a', VomStyle::default());
        grid.cells[1][3] = ('m', VomStyle::default());
        grid.cells[1][4] = ('e', VomStyle::default());
        grid.cells[1][5] = (':', VomStyle::default());

        // DIFFERENT STYLE for input to prevent merging
        let input_style = VomStyle { bold: true, ..VomStyle::default() };
        grid.cells[1][7] = ('_', input_style);
        grid.cells[1][8] = ('_', input_style);
        grid.cells[1][9] = ('_', input_style);

        // Cursor on the underscores at (1,8)
        let components = analyze(&grid, (1, 8));

        for c in &components {
            println!("Component: {:?} ID={} text='{}' at x={}", c.role, c.id, c.text, c.bounds.x);
        }

        let input = components.iter().find(|c| c.role == Role::Input).expect("Should find input");
        assert_eq!(input.bounds.x, 7);
        assert_eq!(input.bounds.width, 3);

        let label = components.iter().find(|c| c.role == Role::StaticText).expect("Should find label");
        assert_eq!(label.text, "Name:");
        assert_eq!(label.bounds.x, 1);
    }
}
