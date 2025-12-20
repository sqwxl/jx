use crossterm::{cursor, queue, style::PrintStyledContent};
use std::io::Write;

use crate::style::{STYLE_HELP_BORDER, STYLE_HELP_DESC, STYLE_HELP_KEY};

const HELP_CONTENT: &[(&str, &str)] = &[
    ("Navigation", ""),
    ("h/←", "Move out"),
    ("j/↓", "Move down"),
    ("k/↑", "Move up"),
    ("l/→", "Move in"),
    ("", ""),
    ("Scrolling", ""),
    ("y/e", "Scroll line up/down"),
    ("u/d", "Scroll half page"),
    ("b/f", "Scroll full page"),
    ("g/G", "Go to top/bottom"),
    ("</>", "Scroll left/right"),
    ("", ""),
    ("Actions", ""),
    ("Space", "Toggle fold"),
    ("/", "Search"),
    ("n/N", "Next/prev match"),
    ("Esc", "Clear search"),
    ("", ""),
    ("Output", ""),
    ("Enter", "Output selection"),
    ("o/O", "Output compact"),
    ("c/C", "Copy to clipboard"),
    ("r/R", "Copy compact"),
    ("", ""),
    ("Other", ""),
    ("w", "Toggle line wrap"),
    ("?", "Show this help"),
    ("q", "Quit"),
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
        PrintStyledContent(STYLE_HELP_BORDER.apply(format!("╭{}╮", "─".repeat(box_width - 2))))
    )?;

    // Draw content lines
    for (i, (key, desc)) in HELP_CONTENT.iter().enumerate() {
        let y = start_y + 1 + i;
        queue!(
            out,
            cursor::MoveTo(start_x as u16, y as u16),
            PrintStyledContent(STYLE_HELP_BORDER.apply("│ "))
        )?;

        if desc.is_empty() {
            // Section header
            let padding = box_width - 4 - key.chars().count();
            queue!(
                out,
                PrintStyledContent(STYLE_HELP_DESC.apply(*key)),
                PrintStyledContent(STYLE_HELP_BORDER.apply(format!("{} │", " ".repeat(padding))))
            )?;
        } else {
            // Key-value pair
            let padding = box_width - 4 - key.chars().count() - 3 - desc.chars().count();
            queue!(
                out,
                PrintStyledContent(STYLE_HELP_KEY.apply(*key)),
                PrintStyledContent(STYLE_HELP_DESC.apply(format!(" - {}", desc))),
                PrintStyledContent(STYLE_HELP_BORDER.apply(format!("{} │", " ".repeat(padding))))
            )?;
        }
    }

    // Draw bottom border
    queue!(
        out,
        cursor::MoveTo(start_x as u16, (start_y + box_height - 1) as u16),
        PrintStyledContent(STYLE_HELP_BORDER.apply(format!("╰{}╯", "─".repeat(box_width - 2))))
    )?;

    Ok(())
}
