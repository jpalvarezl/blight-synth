use crate::id::InstrumentId;

pub enum PlayerCommand {
    PlayNote {
        instrument_id: InstrumentId,
        note: u8,
        velocity: u8,
    },
    StopNote {
        instrument_id: InstrumentId,
    },
    ChangeInstrumentParam {
        instrument_id: InstrumentId,
        parameter: InstrumentParameter,
    },
}

pub enum InstrumentParameter {}
