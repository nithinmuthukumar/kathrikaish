use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use rodio::{
    source::{Buffered, SamplesConverter},
    Decoder, OutputStream, OutputStreamHandle, Source,
};
use shrs::anyhow::Result;
use shrs::anyhow::{anyhow, Context as AnyhowContext};
use shrs::prelude::*;
use shrs_command_timer::CommandTimerState;

pub type ShrsAudio = Buffered<SamplesConverter<Decoder<BufReader<File>>, f32>>;

pub struct AudioStreamState {
    stream_handle: OutputStreamHandle,
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
                    source.convert_samples::<f32>().buffered(),
                );
            }
        }
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        Ok(AudioStreamState {
            stream_handle,
            _stream,
            audios,
        })
    }
    pub fn play_sound(&self, file_name: &str) -> anyhow::Result<()> {
        // Check if the audio file exists in the HashMap
        let audio_buffer = self
            .audios
            .get(file_name)
            .with_context(|| format!("Audio file '{}' not found", file_name))?;

        // Attempt to play the audio
        self.stream_handle
            .play_raw(audio_buffer.to_owned())
            .with_context(|| "Failed to play audio")?;

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
    sh: &Shell,
    ctx: &mut Context,
    rt: &mut Runtime,
    ac_ctx: &AfterCommandCtx,
) -> anyhow::Result<()> {
    if let Some(stream) = ctx.state.get::<AudioStreamState>() {
        if let Some(timer) = ctx.state.get::<CommandTimerState>() {
            if timer.command_time().map_or_else(|| 0, |t| t.as_secs()) > 3 {
                stream.play_sound("complete.mp3")?;
            }
        }
    }
    Ok(())
}

pub fn switch_mode_sfx(
    sh: &Shell,
    ctx: &mut Context,
    rt: &mut Runtime,
    lms_ctx: &LineModeSwitchCtx,
) -> anyhow::Result<()> {
    if let Some(stream) = ctx.state.get::<AudioStreamState>() {
        stream.play_sound("mode_switch.wav")?;
    }

    // match lms_ctx.line_mode {
    //     shrs::readline::LineMode::Insert => todo!(),
    //     shrs::readline::LineMode::Normal => todo!(),
    // }
    Ok(())
}
pub fn startup_sfx(
    sh: &Shell,
    ctx: &mut Context,
    rt: &mut Runtime,
    lms_ctx: &StartupCtx,
) -> anyhow::Result<()> {
    if let Some(stream) = ctx.state.get::<AudioStreamState>() {
        stream.play_sound("meow.wav")?;
    }

    // match lms_ctx.line_mode {
    //     shrs::readline::LineMode::Insert => todo!(),
    //     shrs::readline::LineMode::Normal => todo!(),
    // }
    Ok(())
}
