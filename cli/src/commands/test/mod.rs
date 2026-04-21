mod ios;
mod platform;

use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use crate::{
    error::{TestError, WKCliError},
    output::colored_println,
};

use platform::PlatformBackend;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Platform {
    Ios,
}

#[derive(Debug, Args)]
pub struct Test {
    /// Target platform.
    #[arg(long, value_enum)]
    pub platform: Platform,

    /// Target device by ID (UDID for iOS, serial for Android).
    /// Falls back to the most recently booted device when omitted.
    #[arg(long)]
    pub device: Option<String>,

    /// Override the WDA /source fetch timeout (seconds). Raise when working
    /// with large accessibility trees where layout-map exceeds the default 8s.
    #[arg(long)]
    pub source_timeout: Option<u32>,

    #[command(subcommand)]
    pub subcommand: TestSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum TestSubcommand {
    /// One-shot: boot simulator, build WDA, install app, start WDA, launch app.
    Setup {
        /// Path to .app bundle or bundle identifier.
        app: String,
        /// Automation server HTTP port.
        #[arg(long, default_value_t = 8100)]
        port: u16,
    },
    /// Start the automation server (WebDriverAgent on iOS).
    Start {
        #[arg(long, default_value_t = 8100)]
        port: u16,
    },
    /// Print booted device info and screen bounds.
    Status,
    /// Verify the device, automation server, and host toolchain are healthy.
    Doctor,
    /// Kill stale automation server processes and clear cached session state.
    Teardown,
    /// Re-pin and activate the target app so it is in the foreground.
    Activate {
        /// Bundle identifier. Defaults to the script's configured default.
        bundle_id: Option<String>,
    },
    /// Full structured JSON snapshot of the current screen.
    LayoutMap,
    /// Get the current screen title.
    Title,
    /// Tap at logical coordinates.
    Tap { x: f64, y: f64 },
    /// Tap an element by accessibility label. Waits for the UI to settle and
    /// returns the resulting layout-map unless --no-wait is set.
    TapOn {
        label: String,
        /// Wait until this label appears before returning.
        #[arg(long)]
        wait_for: Option<String>,
        /// Seconds to wait for the post-tap transition.
        #[arg(long, default_value_t = 5)]
        timeout: u32,
        /// Fire-and-forget: skip the wait and don't return a layout-map.
        #[arg(long)]
        no_wait: bool,
    },
    /// Swipe between two points.
    Swipe {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        #[arg(long, default_value_t = 10)]
        steps: u32,
        #[arg(long, default_value_t = 50)]
        step_ms: u32,
    },
    /// Scroll the screen.
    Scroll {
        #[command(subcommand)]
        kind: ScrollKind,
    },
    /// Type text into the focused field.
    Type { text: String },
    /// Block until a label appears, or (with --stable) until the UI stops changing.
    Wait {
        /// Accessibility label to wait for. Required unless --stable is set.
        label: Option<String>,
        /// Wait for UI stability instead of a specific label.
        #[arg(long)]
        stable: bool,
        /// Max seconds to wait.
        #[arg(long, default_value_t = 10)]
        timeout: u32,
    },
    /// Find an element by label.
    FindElement { label: String },
    /// Hit-test the element at a coordinate.
    HitTest { x: f64, y: f64 },
    /// Dump the accessibility tree as JSON.
    Describe {
        depth: Option<u32>,
        #[arg(short, long)]
        interactive: bool,
    },
    /// Capture a PNG screenshot.
    Screenshot { output: String },
}

#[derive(Debug, Subcommand)]
pub enum ScrollKind {
    /// Scroll up (reveals content above).
    Up {
        #[arg(long)]
        x: Option<f64>,
        #[arg(long)]
        from: Option<f64>,
        #[arg(long)]
        to: Option<f64>,
    },
    /// Scroll down (reveals content below).
    Down {
        #[arg(long)]
        x: Option<f64>,
        #[arg(long)]
        from: Option<f64>,
        #[arg(long)]
        to: Option<f64>,
    },
    /// Scroll until the given label is visible.
    To {
        label: String,
        #[arg(long, default_value_t = 10)]
        max_swipes: u32,
    },
}

impl Test {
    pub async fn handle_command(&self) -> Result<bool, WKCliError> {
        let backend = self.backend();
        dispatch(backend.as_ref(), &self.subcommand).await
    }

    fn backend(&self) -> Box<dyn PlatformBackend> {
        match self.platform {
            Platform::Ios => Box::new(ios::IosBackend::new(
                self.device.clone(),
                self.source_timeout,
            )),
        }
    }
}

async fn dispatch(
    backend: &dyn PlatformBackend,
    subcommand: &TestSubcommand,
) -> Result<bool, WKCliError> {
    match subcommand {
        TestSubcommand::Setup { app, port } => backend.setup(app, *port).await?,
        TestSubcommand::Start { port } => backend.wda_start(*port).await?,
        TestSubcommand::Status => backend.status().await?,
        TestSubcommand::Doctor => backend.health_check().await?,
        TestSubcommand::Teardown => backend.cleanup().await?,
        TestSubcommand::Activate { bundle_id } => {
            backend.ensure_foreground_app(bundle_id.as_deref()).await?
        }
        TestSubcommand::LayoutMap => print_json(&backend.layout_map().await?)?,
        TestSubcommand::Title => println!("{}", backend.screen_title().await?),
        TestSubcommand::Tap { x, y } => backend.tap(*x, *y).await?,
        TestSubcommand::TapOn {
            label,
            wait_for,
            timeout,
            no_wait,
        } => {
            if *no_wait {
                backend.tap_element(label).await?;
            } else {
                let layout = backend
                    .tap_and_wait(label, wait_for.as_deref(), *timeout)
                    .await?;
                print_json(&layout)?;
            }
        }
        TestSubcommand::Swipe {
            x1,
            y1,
            x2,
            y2,
            steps,
            step_ms,
        } => {
            backend
                .swipe(*x1, *y1, *x2, *y2, *steps, *step_ms)
                .await?
        }
        TestSubcommand::Scroll { kind } => match kind {
            ScrollKind::Up { x, from, to } => backend.scroll_up(*x, *from, *to).await?,
            ScrollKind::Down { x, from, to } => backend.scroll_down(*x, *from, *to).await?,
            ScrollKind::To { label, max_swipes } => {
                backend.scroll_to_visible(label, *max_swipes).await?
            }
        },
        TestSubcommand::Type { text } => backend.type_text(text).await?,
        TestSubcommand::Wait {
            label,
            stable,
            timeout,
        } => match (label, *stable) {
            (Some(l), false) => backend.wait_for(l, *timeout).await?,
            (None, true) => backend.wait_for_stable(*timeout).await?,
            (Some(_), true) => {
                return Err(WKCliError::TestError(TestError::InvalidInvocation(
                    "wait: --stable is mutually exclusive with a label argument".into(),
                )));
            }
            (None, false) => {
                return Err(WKCliError::TestError(TestError::InvalidInvocation(
                    "wait: provide a <label> or pass --stable".into(),
                )));
            }
        },
        TestSubcommand::FindElement { label } => print_json(&backend.find_element(label).await?)?,
        TestSubcommand::HitTest { x, y } => print_json(&backend.describe_point(*x, *y).await?)?,
        TestSubcommand::Describe {
            depth,
            interactive,
        } => print_json(&backend.describe(*depth, *interactive).await?)?,
        TestSubcommand::Screenshot { output } => backend.screenshot(output).await?,
    }
    Ok(true)
}

fn print_json<T: Serialize>(value: &T) -> Result<(), WKCliError> {
    let rendered = serde_json::to_string_pretty(value).map_err(|e| {
        WKCliError::TestError(TestError::InvalidScriptOutput {
            subcommand: "<serialize>".into(),
            reason: e.to_string(),
        })
    })?;
    colored_println!("{rendered}");
    Ok(())
}
