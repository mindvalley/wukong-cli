pub enum HotKey {
    SelectNamespace,
    Quit,
}

impl HotKey {
    pub fn keycode(&self) -> char {
        match self {
            HotKey::SelectNamespace => 'n',
            HotKey::Quit => 'q',
        }
    }

    pub fn desc(&self) -> &'static str {
        match self {
            HotKey::SelectNamespace => "Select namespace",
            HotKey::Quit => "Quit",
        }
    }
}
