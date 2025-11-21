pub enum AudioControllerCommand {
    Play,
    Pause,
    Stop,
    Next,
    Previous,
    SetVolume(f32),
    SetSpeed(f32),
    SetProgress(u64),
    Load(String),
}