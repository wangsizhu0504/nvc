use std::io::Read;
use std::{fmt::Write};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use reqwest::blocking::Response;

const PROGRESS_CHARS: &str = if cfg!(windows) {
    "█▓▒░  "
} else {
    "█▉▊▋▌▍▎▏  "
};

const INDICATIF_PROGRESS_TEMPLATE: &str = if cfg!(windows) {
    // Do not use a spinner on Windows since the default console cannot display
    // the characters used for the spinner
    "{elapsed_precise:.bold} ▐{wide_bar:.blue/white.dim}▌ {bytes}/{total_bytes} {percent:.bold}% ({eta})"
} else {
    "{spinner:.green.bold} [{elapsed_precise:.bold}] ▕{wide_bar:.blue/white.dim}▏ {bytes}/{total_bytes} {percent:.bold}%  ({eta})"
};


pub struct ResponseProgress {
    progress: Option<ProgressBar>,
    response: Response,
}

fn make_progress_bar(size: u64) -> ProgressBar {
    let pb = ProgressBar::new(size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(INDICATIF_PROGRESS_TEMPLATE)
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .progress_chars(PROGRESS_CHARS),
    );
    
    pb
}

impl ResponseProgress {
    pub fn new(response: Response) -> Self {
        Self {
            progress: response
                .content_length()
                .map(|len| make_progress_bar(len)),
            response,
        }
    }
    
    pub fn finish(&self) {
        if let Some(ref bar) = self.progress {
            bar.finish_with_message("downloaded");
        }
    }
}

impl Read for ResponseProgress {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = self.response.read(buf)?;
        
        if let Some(ref bar) = self.progress {
            bar.inc(size as u64);
        }
        
        Ok(size)
    }
}

impl Drop for ResponseProgress {
    fn drop(&mut self) {
        self.finish();
    }
}
