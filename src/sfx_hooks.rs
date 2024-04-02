use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use rodio::{
    source::{Buffered, SamplesConverter},
    Decoder, OutputStream, OutputStreamHandle, Sink, Source,
};
use shrs::anyhow::Result;
use shrs::anyhow::{anyhow, Context as AnyhowContext};
use shrs::prelude::*;
use shrs_command_timer::CommandTimerState;

pub type ShrsAudio = Buffered<Decoder<BufReader<File>>>;

pub struct AudioStreamState {
    sink: Sink,
    // Need to store so it doesn't go out of scope
    _stream: OutputStream,

    audios: HashMap<String, ShrsAudio>,
}
impl AudioStreamState {
    pub fn new(config_dir: &PathBuf) -> Result<Self> {
        let mut audios = HashMap::new();
        for file in fs::read_dir(config_dir.join("audio"))? {
            let p = file?.path();

            let file = BufReader::new(File::open(p.clone())?);
            if let Ok(source) = Decoder::new(file) {
                audios.insert(
                    p.file_name()
                        .ok_or_else(|| anyhow!("No filename"))?
                        .to_string_lossy()
                        .to_string(),
                    source.buffered(),
                );
            }
        }
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        Ok(AudioStreamState {
            sink: Sink::try_new(&stream_handle)?,
            _stream,
            audios,
        })
    }
    pub fn play_sound(&self, file_name: &str, volume: f32) -> anyhow::Result<()> {
        self.sink.set_volume(volume);

        // Check if the audio file exists in the HashMap
        let audio_buffer = self
            .audios
            .get(file_name)
            .with_context(|| format!("Audio file '{}' not found", file_name))?;

        // Attempt to play the audio

        self.sink.append(audio_buffer.to_owned());

        Ok(())
    }
}

pub struct AudioPlugin;
impl Plugin for AudioPlugin {
    fn init(&self, shell: &mut ShellConfig) -> anyhow::Result<()> {
        let state = AudioStreamState::new(&shell.config_dir)?;
        shell.state.insert(state);

        Ok(())
    }
}

pub fn command_finish_sfx(
    _sh: &Shell,
    ctx: &mut Context,
    _rt: &mut Runtime,
    ac_ctx: &AfterCommandCtx,
) -> anyhow::Result<()> {
    if let Some(stream) = ctx.state.get::<AudioStreamState>() {
        match ac_ctx.cmd_output.status.success() {
            true => stream.play_sound("success.wav", 0.3)?,
            false => stream.play_sound("error.wav", 0.3)?,
        };
    }
    Ok(())
}

pub fn switch_mode_sfx(
    _sh: &Shell,
    ctx: &mut Context,
    _rt: &mut Runtime,
    lms_ctx: &LineModeSwitchCtx,
) -> anyhow::Result<()> {
    //     if let Some(stream) = ctx.state.get::<AudioStreamState>() {
    //         match lms_ctx.line_mode {
    //             shrs::readline::LineMode::Insert => stream.play_sound("on.wav", 0.5)?,
    //             shrs::readline::LineMode::Normal => stream.play_sound("off.wav", 0.5)?,
    //         };
    //     }
    //
    Ok(())
}
pub fn startup_sfx(
    _sh: &Shell,
    ctx: &mut Context,
    _rt: &mut Runtime,
    _lms_ctx: &StartupCtx,
) -> anyhow::Result<()> {
    if let Some(stream) = ctx.state.get::<AudioStreamState>() {
        stream.play_sound("meow.wav", 0.5)?;
    }

    Ok(())
}
