use crate::LauncherListItem;

/// Trait defining the interface for any launcher UI implementation
pub trait LauncherUI {
    /// Show the launcher UI
    fn show(&self);
    
    /// Hide the launcher UI
    fn hide(&self);
    
    /// Update the UI with new search results
    fn set_results(&mut self, results: &[Box<dyn LauncherListItem>]);
    
    /// Check if the UI is currently visible
    fn is_visible(&self) -> bool;
}

/// Events that the UI can send to the controller
#[derive(Debug, Clone)]
pub enum UIEvent {
    /// User typed in the search box
    SearchChanged(String),
    /// User selected an item (by index)
    ItemSelected(usize),
    /// User activated an item (pressed Enter or clicked)
    ItemActivated(usize),
    /// User requested to close the launcher (Escape key)
    CloseRequested,
}

/// Events that the controller can send to the UI
#[derive(Debug, Clone)]
pub enum ControllerEvent {
    /// Results have been updated
    ResultsUpdated,
    /// An error occurred
    Error(String),
    /// Close the UI
    Close,
}