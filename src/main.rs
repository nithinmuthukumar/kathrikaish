use ::crossterm::style::Color;
use shrs_cd_stack::CdStackPlugin;
use shrs_command_timer::CommandTimerPlugin;
use shrs_insulter::InsulterPlugin;
use shrs_mux::MuxPlugin;
use shrs_output_capture::OutputCapturePlugin;
use shrs_presence::PresencePlugin;
use shrs_run_context::RunContextPlugin;
use std::{fs, path::PathBuf, process::Command};

use shrs::{
    anyhow,
    history::FileBackedHistory,
    keybindings,
    prelude::{
        builtin_cmdname_action, cmdname_action, cmdname_pred, styled, styled_buf::StyledBuf, Alias,
        Builtins, Context, DefaultCompleter, DefaultMenu, Env, HookFn, Hooks, LineBuilder, LineCtx,
        LineMode, Pred, Prompt, Rule, Runtime, Shell, StartupCtx, Stylize,
    },
    prompt::{hostname, top_pwd},
    ShellBuilder,
};

struct KPrompt;

#[allow(unused_variables)]
impl Prompt for KPrompt {
    fn prompt_left(&self, line_ctx: &mut LineCtx) -> StyledBuf {
        let indicator = match line_ctx.mode() {
            LineMode::Insert => String::from("🍆").cyan(),
            LineMode::Normal => String::from("💦").yellow(),
        };

        // let home = std::env::var("HOME").unwrap();
        // let mut wd = std::env::current_dir().unwrap();
        // if let Ok(p) = wd.strip_prefix(home) {
        //     wd = PathBuf::from(p);
        // }

        styled!(@(green)"╭─ ",@(blue)" ", @(blue,bold)top_pwd(), " ", @(yellow)" master ", "\n",@(green)"╰─ ", indicator," ")
    }

    fn prompt_right(&self, line_ctx: &mut LineCtx) -> StyledBuf {
        styled!()
    }
}

fn main() {
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
        .build()
        .expect("Could not build line");

    let startup_msg: HookFn<StartupCtx> = |_sh: &Shell,
                                           ctx: &mut Context,
                                           _sh_rt: &mut Runtime,
                                           _ctx: &StartupCtx|
     -> anyhow::Result<()> {
        let welcome_str = format!("\n\n\t\tHello Nithin\n\n");

        ctx.out.print_buf(styled!(@(dark_red)welcome_str))?;
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
        .with_plugin(CdStackPlugin)
        .with_plugin(InsulterPlugin::default())
        .with_plugin(PresencePlugin)
        .build()
        .expect("Could not build shell");

    match shell.run() {
        _ => (),
    }
}
