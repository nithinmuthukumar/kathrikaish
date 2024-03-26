use chrono::Local;
use crossterm::style::{Attribute, Color, Stylize};
use shrs::prelude::styled_buf::StyledBuf;
use shrs::prelude::{styled_buf, LineCtx, LineMode, Prompt};
use shrs::prompt::top_pwd;
use shrs_cd_tools::git::{self, commits_ahead_remote, commits_behind_remote};
use shrs_command_timer::CommandTimerState;
use shrs_output_capture::OutputCaptureState;

pub enum Showing {
    Time,
    Date,
    Spotify,
    Strava,
}
pub struct KPromptState {
    showing: usize,
}
impl KPromptState {
    pub fn cycle(&mut self) {
        self.showing = (self.showing + 1) % 4;
    }
}

pub struct KPrompt;

#[allow(unused_variables)]
impl Prompt for KPrompt {
    fn prompt_left(&self, line_ctx: &LineCtx) -> StyledBuf {
        let indicator = match line_ctx.mode() {
            LineMode::Insert => String::from("🍆").cyan(),
            LineMode::Normal => String::from("💦").yellow(),
        };

        // let home = std::env::var("HOME").unwrap();
        // let mut wd = std::env::current_dir().unwrap();
        // if let Ok(p) = wd.strip_prefix(home) {
        //     wd = PathBuf::from(p);
        // }
        let git_branch = git::branch().map_or(String::new(), |s| format!(" {s} "));
        let commits_behind = commits_behind_remote().map_or(String::new(), |s| {
            if s == 0 {
                String::new()
            } else {
                format!("⇣{s}")
            }
        });

        let commits_ahead = commits_ahead_remote().map_or(String::new(), |s| {
            if s == 0 {
                String::new()
            } else {
                format!("⇡{s}")
            }
        });

        let git_info = git_branch + commits_behind.as_str() + commits_ahead.as_str();

        styled_buf!(
            "╭─ ".green(),
            styled_buf!(" ", top_pwd().bold()).blue(),
            " ",
            git_info.yellow(),
            "\n",
            "╰─ ".green(),
            indicator,
            " "
        )
    }

    fn prompt_right(&self, line_ctx: &LineCtx) -> StyledBuf {
        let time_str = line_ctx
            .ctx
            .state
            .get::<CommandTimerState>()
            .and_then(|x| x.command_time())
            .map(|x| {
                if x.as_secs() < 1 {
                    String::new().blue()
                } else {
                    format!("{:?}s", x.as_secs()).with(Color::Rgb { r: 255, g: 0, b: 0 })
                }
            });
        let status = line_ctx
            .ctx
            .state
            .get::<OutputCaptureState>()
            .unwrap()
            .last_output
            .status
            .code()
            .unwrap_or(-1);

        //Command time in seconds, if it is longer than 0.5 seconds
        //Project Context
        //

        let command_status = if status == 0 {
            styled_buf!("".green())
        } else {
            styled_buf!(status.to_string().red())
        };
        let local_time = Local::now().format("%-I:%M %P").to_string();

        styled_buf!(
            command_status,
            " ",
            time_str,
            " ",
            local_time.dark_cyan().bold(),
            " ".dark_cyan(),
            " ─╮".green(),
            "\n",
            " ".dark_cyan(),
            line_ctx.cb.cursor().to_string().dark_cyan(),
            " ─╯".green()
        )
    }
}
