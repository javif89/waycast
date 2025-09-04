use crate::{LaunchError, LauncherListItem, LauncherPlugin};

struct FileEntry {
    title: String,
}

impl LauncherListItem for ExampleEntry {
    fn title(&self) -> String {
        return self.title.to_string();
    }
    fn description(&self) -> Option<String> {
        return None;
    }

    fn execute(&self) -> Result<(), LaunchError> {
        println!("Sample item clicked: {}", self.title);
        Ok(())
    }
    fn icon(&self) -> String {
        return String::from("vscode");
    }
}

pub struct FileSearchPlugin {}
impl LauncherPlugin for FileSearchPlugin {
    fn name(&self) -> String {
        return String::from("File search");
    }

    fn priority(&self) -> i32 {
        return 900;
    }

    fn description(&self) -> Option<String> {
        None
    }

    fn prefix(&self) -> Option<String> {
        Some(String::from("f"))
    }

    fn by_prefix_only(&self) -> bool {
        false
    }

    fn default_list(&self) -> Vec<Box<dyn LauncherListItem>> {
        Vec::new()
    }

    fn filter(&self, query: &str) -> Vec<Box<dyn LauncherListItem>> {
        self.default_list()
    }
}
