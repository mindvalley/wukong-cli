use inquire::ui::{
    Attributes, Color, ErrorMessageRenderConfig, IndexPrefix, RenderConfig, StyleSheet, Styled,
};

pub fn inquire_render_config() -> RenderConfig {
    RenderConfig {
        prompt_prefix: Styled::new("?").with_style_sheet(
            StyleSheet::new()
                .with_fg(Color::LightCyan)
                .with_attr(Attributes::BOLD),
        ),
        answered_prompt_prefix: Styled::new("❯").with_fg(Color::LightGreen),
        prompt: StyleSheet::empty(),
        default_value: StyleSheet::empty().with_fg(Color::DarkGrey),
        placeholder: StyleSheet::new().with_fg(Color::DarkGrey),
        help_message: StyleSheet::empty()
            .with_fg(Color::LightMagenta)
            .with_attr(Attributes::BOLD),
        text_input: StyleSheet::empty(),
        error_message: ErrorMessageRenderConfig::default_colored().with_prefix(Styled::new("")),
        password_mask: '*',
        answer: StyleSheet::empty()
            .with_fg(Color::LightCyan)
            .with_attr(Attributes::BOLD),
        canceled_prompt_indicator: Styled::new("<canceled>").with_fg(Color::DarkRed),
        highlighted_option_prefix: Styled::new("❯").with_fg(Color::LightCyan),
        scroll_up_prefix: Styled::new("↑"),
        scroll_down_prefix: Styled::new("↓"),
        selected_checkbox: Styled::new("[x]")
            .with_fg(Color::LightGreen)
            .with_attr(Attributes::BOLD),
        unselected_checkbox: Styled::new("[ ]").with_attr(Attributes::BOLD),
        option_index_prefix: IndexPrefix::None,
        option: StyleSheet::empty(),
        selected_option: Some(StyleSheet::new().with_fg(Color::LightCyan)),
        editor_prompt: StyleSheet::new().with_fg(Color::DarkCyan),
    }
}
