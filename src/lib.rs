pub mod drun;
pub mod plugins;
pub mod util;
#[derive(Debug)]
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
    fn name(&self) -> String;
    fn priority(&self) -> i32;
    fn description(&self) -> Option<String>;
    // Prefix to isolate results to only use this plugin
    fn prefix(&self) -> Option<String>;
    // Only search/use this plugin if the prefix was typed
    fn by_prefix_only(&self) -> bool;
    // Actual item searching functions
    fn default_list(&self) -> Vec<Box<dyn LauncherListItem>>;
    fn filter(&self, query: &str) -> Vec<Box<dyn LauncherListItem>>;
}
