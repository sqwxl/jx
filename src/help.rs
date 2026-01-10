use crossterm::{cursor, queue, style::PrintStyledContent};
use std::io::Write;

use crate::style::{styled, STYLE_HELP_BORDER, STYLE_HELP_DESC, STYLE_HELP_KEY};

const HELP_CONTENT: &[(&str, &str)] = &[
    ("Navigation", ""),
    ("h/←", "Move out"),
    ("j/↓", "Move down"),
    ("k/↑", "Move up"),
    ("l/→", "Move in"),
    ("", ""),
    ("Scrolling", ""),
    ("C-y/C-e", "Scroll line up/down"),
    ("u/d", "Scroll half page"),
    ("b/f", "Scroll full page"),
    ("g/G", "Go to top/bottom"),
    ("</>", "Scroll left/right"),
    ("", ""),
    ("Actions", ""),
    ("Space/Enter", "Toggle fold"),
    ("z", "Toggle all folds"),
    ("/", "Search"),
    ("n/N", "Next/prev match"),
    ("Esc", "Clear search"),
    ("", ""),
    ("Output", ""),
    ("o/O", "Output pretty selection/value"),
    ("A-o/A-O", "Output raw selection/value"),
    ("y/Y", "Copy pretty selection/value"),
    ("A-y/A-Y", "Copy raw selection/value"),
    ("", ""),
    ("Other", ""),
    ("w", "Toggle line wrap"),
    ("#", "Toggle line numbers"),
    ("?", "Show this help"),
    ("q/C-c", "Quit"),
];

pub fn render_help<W: Write>(out: &mut W, screen_size: (usize, usize)) -> anyhow::Result<()> {
    // Calculate popup dimensions (use chars().count() for proper Unicode width)
    let content_width = HELP_CONTENT
        .iter()
        .map(|(k, v)| {
            if v.is_empty() {
                k.chars().count()
            } else {
                k.chars().count() + 3 + v.chars().count()
            }
        })
        .max()
        .unwrap_or(20);
    let box_width = content_width + 4; // 2 chars padding on each side
    let box_height = HELP_CONTENT.len() + 2; // +2 for top/bottom borders

    // Center the popup
    let start_x = screen_size.0.saturating_sub(box_width) / 2;
    let start_y = screen_size.1.saturating_sub(box_height) / 2;

    // Draw top border
    queue!(
        out,
        cursor::MoveTo(start_x as u16, start_y as u16),
        PrintStyledContent(styled(
            STYLE_HELP_BORDER,
            format!("╭{}╮", "─".repeat(box_width - 2))
        ))
    )?;

    // Draw content lines
    for (i, (key, desc)) in HELP_CONTENT.iter().enumerate() {
        let y = start_y + 1 + i;
        queue!(
            out,
            cursor::MoveTo(start_x as u16, y as u16),
            PrintStyledContent(styled(STYLE_HELP_BORDER, "│ "))
        )?;

        if desc.is_empty() {
            // Section header
            let padding = box_width - 4 - key.chars().count();
            queue!(
                out,
                PrintStyledContent(styled(STYLE_HELP_DESC, *key)),
                PrintStyledContent(styled(
                    STYLE_HELP_BORDER,
                    format!("{} │", " ".repeat(padding))
                ))
            )?;
        } else {
            // Key-value pair
            let padding = box_width - 4 - key.chars().count() - 3 - desc.chars().count();
            queue!(
                out,
                PrintStyledContent(styled(STYLE_HELP_KEY, *key)),
                PrintStyledContent(styled(STYLE_HELP_DESC, format!(" - {}", desc))),
                PrintStyledContent(styled(
                    STYLE_HELP_BORDER,
                    format!("{} │", " ".repeat(padding))
                ))
            )?;
        }
    }

    // Draw bottom border
    queue!(
        out,
        cursor::MoveTo(start_x as u16, (start_y + box_height - 1) as u16),
        PrintStyledContent(styled(
            STYLE_HELP_BORDER,
            format!("╰{}╯", "─".repeat(box_width - 2))
        ))
    )?;

    Ok(())
}
