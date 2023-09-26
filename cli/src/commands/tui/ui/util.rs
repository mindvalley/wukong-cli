use ratatui::style::{Color, Style};

pub fn get_color(
    (is_active, is_hovered): (bool, bool),
    (selected, hovered, inactive): (Color, Color, Color),
) -> Style {
    match (is_active, is_hovered) {
        (true, _) => Style::default().fg(selected),
        (false, true) => Style::default().fg(hovered),
        _ => Style::default().fg(inactive),
    }
}
