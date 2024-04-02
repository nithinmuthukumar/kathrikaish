
use clap::Parser;
use crossterm::style::Color;
use crossterm::{
    style::{ContentStyle, Stylize},
};
use prompt::KPrompt;
use sfx_hooks::{command_finish_sfx, startup_sfx, switch_mode_sfx, AudioPlugin};
use shrs::shell::{set_working_dir, ShellBuilder};
use shrs_cd_stack::{CdStackPlugin, CdStackState};
use shrs_cd_tools::{
    DirParsePlugin,
};
use shrs_command_timer::{CommandTimerPlugin};
use shrs_insulter::prelude::*;
use shrs_mux::MuxPlugin;
use shrs_output_capture::{OutputCapturePlugin};
use shrs_presence::PresencePlugin;
use shrs_rhai_completion::CompletionsPlugin;
use shrs_run_context::RunContextPlugin;
use std::{
    fs::{self, File},
    io::Read,
    process::Command,
};

use shrs::{
    anyhow,
    completion::Completer,
    history::FileBackedHistory,
    keybindings,
    prelude::{
        builtin_cmdname_action, cmdname_action, cmdname_pred, Alias, Builtins, Context,
        DefaultCompleter, DefaultMenu, Env, HookFn, Hooks, LineBuilder, Pred, Rule, Runtime, Shell,
        StartupCtx, SyntaxHighlighter,
    },
};

use crate::prompt::KPromptState;
mod prompt;
mod sfx_hooks;

#[derive(Parser, Debug)]
#[command(author="Nithin Muthukumar", version, about = "Nithin's shell", long_about = None)]
struct Args {}

fn main() {
    Args::parse();

    // =-=-= Configuration directory =-=-=
    // Initialize the directory we will be using to hold our configuration and metadata files
    let config_dir = dirs::home_dir().unwrap().as_path().join(".config/shrs");
    // also log when creating dir
    // TODO ignore errors for now (we dont care if dir already exists)
    fs::create_dir_all(config_dir.clone());

    let keybinding = keybindings! {|line|
        "C-l"=>("Clear the screen", {Command::new("clear").spawn().unwrap() }),
        "C-p" => ("Move up one in the command history", {
            if let Some(state) = line.ctx.state.get_mut::<CdStackState>() {
                if let Some(new_path) = state.down() {
                    set_working_dir(line.sh, line.ctx, line.rt, &new_path, false).unwrap();
                }
            }
        }),
        "C-n" => ("Move down one in the command history", {
            if let Some(state) = line.ctx.state.get_mut::<CdStackState>() {
                if let Some(new_path) = state.up() {
                    set_working_dir(line.sh, line.ctx, line.rt, &new_path, false).unwrap();
                }
            }
        }),
        "C-<tab>" => ("Cycle prompt", {
            if let Some(state) = line.ctx.state.get_mut::<KPromptState>(){
                state.cycle();
            }

        })

    };
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
        .with_menu(menu)
        .with_highlighter(SyntaxHighlighter::default())
        .build()
        .expect("Could not build line");

    let startup_msg: HookFn<StartupCtx> = |_sh: &Shell,
                                           _ctx: &mut Context,
                                           _sh_rt: &mut Runtime,
                                           _startup_ctx: &StartupCtx|
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
    hooks.insert(switch_mode_sfx);
    hooks.insert(command_finish_sfx);
    hooks.insert(startup_sfx);

    let mut insult_string = String::new();
    File::open(config_dir.as_path().join("insults.json"))
        .expect("Error opening insult file")
        .read_to_string(&mut insult_string)
        .unwrap();
    let insults: Vec<String> =
        serde_json::from_str(&insult_string).expect("Unable to deserialize JSON");

    let shell = ShellBuilder::default()
        .with_completer(completer)
        .with_readline(readline)
        .with_alias(alias)
        .with_hooks(hooks)
        .with_keybinding(keybinding)
        .with_history(history)
        .with_plugin(AudioPlugin)
        .with_plugin(MuxPlugin::new())
        .with_plugin(OutputCapturePlugin)
        .with_plugin(CommandTimerPlugin)
        .with_plugin(RunContextPlugin::default())
        .with_plugin(CdStackPlugin)
        .with_plugin(CompletionsPlugin)
        .with_plugin(InsulterPlugin::new(
            insults,
            1.,
            false,
            ContentStyle::new().with(Color::DarkRed),
        ))
        .with_plugin(PresencePlugin::new(
            "https://github.com/nithinmuthukumar/kathrikaish".to_string(),
        ))
        .with_plugin(DirParsePlugin::new())
        .build()
        .expect("Could not build shell");

    match shell.run() {
        _ => (),
    }
}
