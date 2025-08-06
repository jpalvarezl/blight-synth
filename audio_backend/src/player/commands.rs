pub enum PlayerCommand {
    PlayNote {
        track_id: usize,
        note: u8,
        velocity: u8,
    },
    StopNote {
        track_id: usize,
    },
}
