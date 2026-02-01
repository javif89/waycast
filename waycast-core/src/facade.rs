use crate::LauncherItem;

pub struct WaycastLauncher {
    db: waycast,
}

impl Default for WaycastLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl WaycastLauncher {
    pub fn new() -> Self {
        WaycastLauncher {}
    }

    pub fn get_default_results(&mut self) -> &Vec<LauncherItem> {
        let mut all_entries = Vec::new();
        for plugin in &self.plugins_show_always {
            all_entries.extend(plugin.default_list());
        }
        for entry in all_entries {
            self.add_current_item(entry);
        }
        &self.current_results
    }

    pub fn search(&mut self, query: &str) -> &Vec<LauncherItem> {
        self.clear_current_results();

        let mut all_entries = Vec::new();
        for plugin in &self.plugins {
            all_entries.extend(plugin.filter(query));
        }
        for entry in all_entries {
            self.add_current_item(entry);
        }

        &self.current_results
    }

    pub fn execute_item(&self, index: usize) -> Result<(), LaunchError> {
        Ok(())
        // if let Some(item) = self.current_results.get(index) {
        //     item.execute()
        // } else {
        //     Err(LaunchError::CouldNotLaunch("Invalid index".into()))
        // }
    }

    pub fn execute_item_by_id(&self, id: &str) -> Result<(), LaunchError> {
        Ok(())
        // if let Some(&index) = self.current_results_by_id.get(id) {
        //     if let Some(item) = self.current_results.get(index) {
        //         item.execute()
        //     } else {
        //         Err(LaunchError::CouldNotLaunch(
        //             "Item index out of bounds".into(),
        //         ))
        //     }
        // } else {
        //     Err(LaunchError::CouldNotLaunch("Item not found".into()))
        // }
    }

    pub fn current_results(&self) -> &Vec<LauncherItem> {
        &self.current_results
    }

    pub fn refresh_plugins(&self) {
        for plugin in &self.plugins {
            plugin.init();
        }
    }
}
