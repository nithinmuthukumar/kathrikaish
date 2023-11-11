use shrs_mux::MuxPlugin;
use std::{path::PathBuf, process::Command};

use shrs::{
    anyhow,
    prelude::{
        keybindings, styled, Alias, Context, Env, HookFn, Hooks, LineBuilder, LineCtx, LineMode,
        Prompt, Runtime, Shell, StartupCtx, StyledBuf, Stylize,
    },
    prompt::top_pwd,
    ShellBuilder,
};

struct KPrompt;

#[allow(unused_variables)]
impl Prompt for KPrompt {
    fn prompt_left(&self, line_ctx: &mut LineCtx) -> StyledBuf {
        let indicator = match line_ctx.mode() {
            LineMode::Insert => String::from("🍆").cyan(),
            LineMode::Normal => String::from("🪝").yellow(),
        };

        let home = std::env::var("HOME").unwrap();
        let mut wd = std::env::current_dir().unwrap();
        if let Ok(p) = wd.strip_prefix(home) {
            wd = PathBuf::from("~").join(p);
        }
        wd.pop();

        styled!(" ", @(red)wd.to_string_lossy().to_string()+"/", @(red,bold)top_pwd(), " ", indicator, " ")
    }

    fn prompt_right(&self, line_ctx: &mut LineCtx) -> StyledBuf {
        styled!()
    }
}

fn main() {
    let keybinding = keybindings! {|_sh,_ctx,_rt| "C-l"=>{Command::new("clear").spawn().unwrap() }};
    let alias = Alias::from_iter([("l", "ls"), ("g", "git"), ("v", "nvim")]);
    let mut env = Env::new();
    env.load().unwrap();

    let readline = LineBuilder::default()
        .with_keybinding(keybinding)
        .with_prompt(KPrompt)
        .build()
        .expect("Could not build line");

    let startup_msg: HookFn<StartupCtx> = |_sh: &Shell,
                                           _sh_ctx: &mut Context,
                                           _sh_rt: &mut Runtime,
                                           _ctx: &StartupCtx|
     -> anyhow::Result<()> {
        let welcome_str = "HI NITHIN";
        println!("{welcome_str}");
        Ok(())
    };
    let mut hooks = Hooks::new();
    hooks.register(startup_msg);

    let shell = ShellBuilder::default()
        .with_readline(readline)
        .with_alias(alias)
        .with_plugin(MuxPlugin)
        .with_hooks(hooks)
        .build()
        .expect("Could not build shell");

    match shell.run() {
        _ => (),
    }
}
