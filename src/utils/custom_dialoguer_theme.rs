use core::fmt;

use dialoguer::{
    console::{style, Style, StyledObject},
    theme::Theme,
};

impl Default for CustomDialoguerTheme {
    fn default() -> CustomDialoguerTheme {
        CustomDialoguerTheme {
            defaults_style: Style::new().for_stderr().cyan(),
            prompt_style: Style::new().for_stderr().bold(),
            prompt_prefix: style("?".to_string()).for_stderr().yellow(),
            prompt_suffix: style("› Space to deselect. Enter to execute".to_string())
                .for_stderr()
                .black()
                .bright(),
            active_item_style: Style::new().for_stderr().white(),
            inactive_item_style: Style::new().for_stderr(),
            active_item_prefix: style("❯".to_string()).for_stderr(),
            inactive_item_prefix: style(" ".to_string()).for_stderr(),
            checked_item_prefix: style("◉".to_string()).for_stderr().green(),
            unchecked_item_prefix: style("◯".to_string()).for_stderr().white(),
        }
    }
}

impl Theme for CustomDialoguerTheme {
    /// Formats a prompt.
    fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
        if !prompt.is_empty() {
            write!(
                f,
                "{} {} ",
                &self.prompt_prefix,
                self.prompt_style.apply_to(prompt)
            )?;
        }

        write!(f, "{}", &self.prompt_suffix)
    }

    /// Formats a multi select prompt item.
    fn format_multi_select_prompt_item(
        &self,
        f: &mut dyn fmt::Write,
        text: &str,
        checked: bool,
        active: bool,
    ) -> fmt::Result {
        let details = match (checked, active) {
            (true, true) => (
                &self.active_item_prefix,
                &self.checked_item_prefix,
                self.active_item_style.apply_to(text),
            ),
            (true, false) => (
                &self.inactive_item_prefix,
                &self.checked_item_prefix,
                self.inactive_item_style.apply_to(text),
            ),
            (false, true) => (
                &self.active_item_prefix,
                &self.unchecked_item_prefix,
                self.active_item_style.apply_to(text),
            ),
            (false, false) => (
                &self.inactive_item_prefix,
                &self.unchecked_item_prefix,
                self.inactive_item_style.apply_to(text),
            ),
        };

        write!(f, "{} {} {}", details.0, details.1, details.2)
    }
}

/// A Custom theme
pub struct CustomDialoguerTheme {
    /// The style for default values
    pub defaults_style: Style,
    /// The style for prompt
    pub prompt_style: Style,
    /// Prompt prefix value and style
    pub prompt_prefix: StyledObject<String>,
    /// Prompt suffix value and style
    pub prompt_suffix: StyledObject<String>,
    /// The style for active items
    pub active_item_style: Style,
    /// The style for inactive items
    pub inactive_item_style: Style,
    /// Active item in select prefix value and style
    pub active_item_prefix: StyledObject<String>,
    /// Inctive item in select prefix value and style
    pub inactive_item_prefix: StyledObject<String>,
    /// Checked item in multi select prefix value and style
    pub checked_item_prefix: StyledObject<String>,
    /// Unchecked item in multi select prefix value and style
    pub unchecked_item_prefix: StyledObject<String>,
}
