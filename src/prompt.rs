use chrono::Local;
use crossterm::style::{Attribute, Color, Stylize};
use shrs::{
    prelude::{styled, styled_buf::StyledBuf, LineCtx, LineMode, Prompt},
    prompt::top_pwd,
};
use shrs_cd_tools::git::{self, commits_ahead_remote, commits_behind_remote};
use shrs_command_timer::CommandTimerState;
use shrs_output_capture::OutputCaptureState;

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

        styled!(
            "╭─ ".with(line_ctx.sh.theme.green),
            styled!(" ", top_pwd().attribute(Attribute::Bold)).with(line_ctx.sh.theme.blue),
            " ",
            git_info.with(line_ctx.sh.theme.yellow),
            "\n",
            "╰─ ".with(line_ctx.sh.theme.green),
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
                    String::new().with(Color::Blue)
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
            styled!("".with(line_ctx.sh.theme.green))
        } else {
            styled!(status.to_string().with(line_ctx.sh.theme.red))
        };
        let local_time = Local::now().format("%-I:%M %P").to_string();
        let lt: Vec<&str> = local_time.split(":").collect();

        styled!(
            command_status,
            " ",
            time_str,
            " ",
            lt[0]
                .with(line_ctx.sh.theme.dark_cyan)
                .attribute(Attribute::Bold),
            ":".attribute(Attribute::SlowBlink),
            lt[1]
                .with(line_ctx.sh.theme.dark_cyan)
                .attribute(Attribute::Bold),
            " ".with(line_ctx.sh.theme.dark_cyan),
            " ─╮".with(line_ctx.sh.theme.green),
            "\n",
            " ".with(line_ctx.sh.theme.dark_cyan),
            line_ctx
                .cb
                .cursor()
                .to_string()
                .with(line_ctx.sh.theme.dark_cyan),
            " ─╯".with(line_ctx.sh.theme.green)
        )
    }
}
