pub mod drun;
pub mod util;
pub enum LaunchError {
    CouldNotLaunch(String),
}

pub trait LauncherListItem {
    fn title(&self) -> String;
    fn description(&self) -> Option<String>;
    fn execute(&self) -> Result<(), LaunchError>;
    fn icon(&self) -> String;
}
