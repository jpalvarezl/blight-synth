pub mod arrangement;
pub mod chains;
pub mod phrases;

#[derive(Debug, PartialEq)]
pub enum CurrentTab {
    Arrangement,
    Chains,
    Phrases,
}

impl CurrentTab {
    /// Cycle to the next tab (left to right)
    pub fn next(&self) -> Self {
        match self {
            CurrentTab::Arrangement => CurrentTab::Chains,
            CurrentTab::Chains => CurrentTab::Phrases,
            CurrentTab::Phrases => CurrentTab::Arrangement,
        }
    }
    
    /// Cycle to the previous tab (right to left)
    pub fn previous(&self) -> Self {
        match self {
            CurrentTab::Arrangement => CurrentTab::Phrases,
            CurrentTab::Chains => CurrentTab::Arrangement,
            CurrentTab::Phrases => CurrentTab::Chains,
        }
    }
}
