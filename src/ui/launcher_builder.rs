use crate::LauncherPlugin;
use crate::ui::WaycastLauncher;
use gtk::Application;
use std::cell::RefCell;
use std::rc::Rc;
pub struct WaycastLauncherBuilder {
    pub plugins: Vec<Box<dyn LauncherPlugin>>,
}

impl WaycastLauncherBuilder {
    pub fn add_plugin<T: LauncherPlugin + 'static>(mut self, plugin: T) -> Self {
        self.plugins.push(Box::new(plugin));
        self
    }

    pub fn initialize(self, app: &Application) -> Rc<RefCell<WaycastLauncher>> {
        WaycastLauncher::create_with_plugins(app, self.plugins)
    }
}
