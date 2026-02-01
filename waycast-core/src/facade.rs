pub struct WaycastLauncher {
    plugins: Vec<Arc<dyn LauncherPlugin>>,
    plugins_show_always: Vec<Arc<dyn LauncherPlugin>>,
    plugins_by_prefix: HashMap<String, Arc<dyn LauncherPlugin>>,
    current_results: Vec<LauncherItem>,
    current_results_by_id: HashMap<String, usize>,
}

impl Default for WaycastLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl WaycastLauncher {
    pub fn new() -> Self {
        WaycastLauncher {
            plugins: Vec::new(),
            plugins_show_always: Vec::new(),
            plugins_by_prefix: HashMap::new(),
            current_results: Vec::new(),
            current_results_by_id: HashMap::new(),
        }
    }

    pub fn add_plugin(mut self, plugin: Box<dyn LauncherPlugin>) -> Self {
        let p: Arc<dyn LauncherPlugin> = plugin.into();
        if !p.by_prefix_only() {
            self.plugins_show_always.push(Arc::clone(&p));
        }

        if let Some(prefix) = p.prefix() {
            self.plugins_by_prefix.insert(prefix, Arc::clone(&p));
        }

        self.plugins.push(p);
        self
    }

    pub fn init(mut self) -> Self {
        for p in &self.plugins {
            p.init();
        }

        self.plugins
            .sort_by_key(|b| std::cmp::Reverse(b.priority()));
        self.plugins_show_always
            .sort_by_key(|b| std::cmp::Reverse(b.priority()));

        self
    }

    fn add_current_item(&mut self, item: LauncherItem) {
        let id = item.clone().id;
        let index = self.current_results.len();
        self.current_results.push(item);
        self.current_results_by_id.insert(id, index);
    }

    fn clear_current_results(&mut self) {
        self.current_results.clear();
        self.current_results_by_id.clear();
    }

    pub fn get_default_results(&mut self) -> &Vec<LauncherItem> {
        self.clear_current_results();
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
