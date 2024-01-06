use ::crossterm::style::Color;
use chrono::Local;
use clap::Parser;
use crossterm::style::Attribute;
use shrs_cd_stack::CdStackPlugin;
use shrs_cd_tools::{
    git::{self, commits_ahead_remote, commits_behind_remote, Git},
    DirParsePlugin, DirParseState,
};
use shrs_command_timer::{CommandTimerPlugin, CommandTimerState};
use shrs_insulter::InsulterPlugin;
use shrs_mux::MuxPlugin;
use shrs_output_capture::{OutputCapturePlugin, OutputCaptureState};
use shrs_presence::PresencePlugin;
use shrs_run_context::RunContextPlugin;
use std::{fs, path::PathBuf, process::Command, thread::sleep, time::Duration};

use shrs::{
    anyhow,
    history::FileBackedHistory,
    keybindings,
    prelude::{
        builtin_cmdname_action, cmdname_action, cmdname_pred, styled, styled_buf::StyledBuf, Alias,
        Builtins, Context, DefaultCompleter, DefaultMenu, Env, HookFn, Hooks, LineBuilder, LineCtx,
        LineMode, Pred, Prompt, Rule, Runtime, Shell, StartupCtx, Stylize, SyntaxHighlighter,
    },
    prompt::{hostname, top_pwd},
    ShellBuilder,
};

struct KPrompt;

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
            " ".with(line_ctx.sh.theme.blue),
            top_pwd()
                .with(line_ctx.sh.theme.blue)
                .attribute(Attribute::Bold),
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
                    format!("{:?}s", x.as_secs()).with(Color::Blue)
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

        //COLOR NOT PRINTING
        let command_status = if status == 0 {
            styled!("".with(line_ctx.sh.theme.green))
        } else {
            styled!(status.to_string().with(line_ctx.sh.theme.red))
        };
        let local_time = Local::now();
        let formatted_time = local_time.format("%-I:%M %P").to_string();
        styled!(
            command_status,
            " ",
            time_str,
            " ",
            formatted_time
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
#[derive(Parser, Debug)]
#[command(author="Nithin Muthukumar", version, about = "Nithin's shell", long_about = None)]
struct Args {}

fn main() {
    let args = Args::parse();

    // =-=-= Configuration directory =-=-=
    // Initialize the directory we will be using to hold our configuration and metadata files
    let config_dir = dirs::home_dir().unwrap().as_path().join(".config/shrs");
    // also log when creating dir
    // TODO ignore errors for now (we dont care if dir already exists)
    fs::create_dir_all(config_dir.clone());

    let keybinding = keybindings! {|_sh,_ctx,_rt| "C-l"=>("Clear the screen", {Command::new("clear").spawn().unwrap() })};
    let alias = Alias::from_iter([("l", "ls"), ("g", "git"), ("v", "nvim")]);
    let mut env = Env::default();
    env.load().unwrap();
    env.set("USER", "Nithin");

    env.set("SHELL", "/Users/nithin/.cargo/bin/kathrikaish");
    env.set("SHELL_NAME", "kathrikaish");

    let builtins = Builtins::default();

    // =-=-= Completion =-=-=
    // Get list of binaries in path and initialize the completer to autocomplete command names
    let path_string = env.get("PATH").unwrap().to_string();
    let mut completer = DefaultCompleter::default();
    completer.register(Rule::new(
        Pred::new(cmdname_pred),
        Box::new(cmdname_action(path_string)),
    ));
    completer.register(Rule::new(
        Pred::new(cmdname_pred),
        Box::new(builtin_cmdname_action(&builtins)),
    ));

    // =-=-= Menu =-=-=-=
    let menu = DefaultMenu::default();

    let readline = LineBuilder::default()
        .with_prompt(KPrompt)
        .with_completer(completer)
        .with_menu(menu)
        .with_highlighter(SyntaxHighlighter::default())
        .build()
        .expect("Could not build line");

    let startup_msg: HookFn<StartupCtx> = |_sh: &Shell,
                                           ctx: &mut Context,
                                           _sh_rt: &mut Runtime,
                                           _ctx: &StartupCtx|
     -> anyhow::Result<()> {
        Command::new("neofetch").spawn()?.wait()?;

        Ok(())
    };
    // =-=-= History =-=-=
    // Use history that writes to file on disk
    let history_file = config_dir.as_path().join("history");
    let history = FileBackedHistory::new(history_file).expect("Could not open history file");

    let mut hooks = Hooks::new();
    hooks.insert(startup_msg);

    let shell = ShellBuilder::default()
        .with_readline(readline)
        .with_alias(alias)
        .with_hooks(hooks)
        .with_keybinding(keybinding)
        .with_history(history)
        .with_plugin(MuxPlugin::new())
        .with_plugin(OutputCapturePlugin)
        .with_plugin(CommandTimerPlugin)
        .with_plugin(RunContextPlugin::default())
        // .with_plugin(CdStackPlugin)
        .with_plugin(InsulterPlugin::default())
        .with_plugin(PresencePlugin)
        // .with_plugin(DirParsePlugin::new())
        .build()
        .expect("Could not build shell");

    match shell.run() {
        _ => (),
    }
}
