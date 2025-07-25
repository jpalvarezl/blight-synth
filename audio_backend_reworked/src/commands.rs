pub enum Command {
    PlayNote { note: u8, velocity: f32 },
    StopNote { note: u8 },
    SetParameter { param: String, value: f32 },
    // Add more commands as needed.
}
