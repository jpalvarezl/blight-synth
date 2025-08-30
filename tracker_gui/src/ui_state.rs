use std::collections::HashMap;

#[derive(Default)]
pub struct UiState {
    pub phrases_note: HashMap<(usize, usize), String>,
    pub phrases_vol: HashMap<(usize, usize), String>,
    pub chains_phrase: HashMap<(usize, usize), String>,
    pub arrangement_chain: HashMap<(usize, usize), String>,
}
