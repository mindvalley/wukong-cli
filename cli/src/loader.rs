use indicatif::{ProgressBar, ProgressStyle};

pub fn new_spinner() -> ProgressBar {
    let steps = 1_000_000;
    let progress_bar = ProgressBar::new(steps);
    progress_bar.set_style(ProgressStyle::default_spinner());

    progress_bar.enable_steady_tick(std::time::Duration::from_millis(80));
    progress_bar
}

pub fn new_progress_bar(steps: u64) -> ProgressBar {
    let progress_bar = ProgressBar::new(steps);
    progress_bar.set_style(ProgressStyle::default_bar());

    progress_bar
}
