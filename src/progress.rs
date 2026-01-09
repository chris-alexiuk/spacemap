use indicatif::{ProgressBar, ProgressStyle};

pub struct ScanProgress {
    bar: ProgressBar,
    enabled: bool,
}

impl ScanProgress {
    pub fn new(enabled: bool) -> Self {
        if !enabled {
            return Self {
                bar: ProgressBar::hidden(),
                enabled: false,
            };
        }

        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );

        Self { bar, enabled: true }
    }

    pub fn update(&self, files: u64, bytes: u64, current_path: &str) {
        if self.enabled {
            // Truncate path if too long
            let display_path = if current_path.len() > 60 {
                format!("...{}", &current_path[current_path.len() - 57..])
            } else {
                current_path.to_string()
            };

            self.bar.set_message(format!(
                "{} files | {} | {}",
                files,
                humansize::format_size(bytes, humansize::BINARY),
                display_path
            ));
            self.bar.tick();
        }
    }

    pub fn finish(&self) {
        if self.enabled {
            self.bar.finish_and_clear();
        }
    }
}
