pub mod arrangement;
pub mod chains;
pub mod phrases;

#[derive(Debug, PartialEq)]
pub enum CurrentTab {
    Arrangement,
    Chains,
    Phrases,
}
