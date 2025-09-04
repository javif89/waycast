pub mod drun;
pub mod plugins;
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

pub trait LauncherPlugin {
    fn name() -> String;
    fn priority() -> i32;
    fn description() -> Option<String>;
    // Prefix to isolate results to only use this plugin
    fn prefix() -> Option<String>;
    // Only search/use this plugin if the prefix was typed
    fn by_prefix_only() -> bool;
    // Actual item searching functions
    fn default_list() -> Vec<Box<dyn LauncherListItem>>;
    fn filter(query: &str) -> Vec<Box<dyn LauncherListItem>>;
}
