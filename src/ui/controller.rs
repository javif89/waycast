use super::gtk::GtkLauncherUI;
use super::traits::{LauncherUI, UIEvent};
use crate::{LaunchError, launcher::WaycastLauncher};
use gio::prelude::ApplicationExt;
use gtk::glib;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver};

/// Controller that coordinates between the core launcher and UI
pub struct LauncherController {
    launcher: Rc<RefCell<WaycastLauncher>>,
    ui: GtkLauncherUI,
    event_receiver: Receiver<UIEvent>,
    app: gtk::Application,
}

impl LauncherController {
    pub fn new(launcher: WaycastLauncher, mut ui: GtkLauncherUI, app: gtk::Application) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();

        // Set up the event sender in the UI
        ui.set_event_sender(event_sender);

        Self {
            launcher: Rc::new(RefCell::new(launcher)),
            ui,
            event_receiver,
            app,
        }
    }

    pub fn initialize(&mut self) {
        // Populate with default results
        let mut launcher = self.launcher.borrow_mut();
        let results = launcher.get_default_results();
        self.ui.set_results(results);
    }

    pub fn show(&self) {
        self.ui.show();
    }

    pub fn handle_event(&mut self, event: UIEvent) -> Result<(), LaunchError> {
        match event {
            UIEvent::SearchChanged(query) => {
                let mut launcher = self.launcher.borrow_mut();
                let results = if query.trim().is_empty() {
                    launcher.get_default_results()
                } else {
                    launcher.search(&query)
                };
                self.ui.set_results(results);
            }

            UIEvent::ItemActivated(index) => match self.launcher.borrow().execute_item(index) {
                Ok(_) => {
                    // Exit the application completely instead of just hiding
                    self.app.quit();
                }
                Err(e) => {
                    eprintln!("Failed to launch item: {:?}", e);
                    return Err(e);
                }
            },

            UIEvent::ItemSelected(_index) => {
                // Handle selection change if needed
                // For now, this is just for keyboard navigation
            }

            UIEvent::CloseRequested => {
                // Exit the application completely instead of just hiding
                self.app.quit();
            }
        }

        Ok(())
    }

    pub fn process_events(&mut self) -> Result<(), LaunchError> {
        while let Ok(event) = self.event_receiver.try_recv() {
            self.handle_event(event)?;
        }
        Ok(())
    }

    pub fn run(mut self) {
        self.initialize();
        self.show();

        // Set up periodic event processing using glib's idle callback
        let controller = Rc::new(RefCell::new(self));
        let controller_clone = controller.clone();

        glib::idle_add_local(move || {
            if let Ok(mut ctrl) = controller_clone.try_borrow_mut() {
                if let Err(e) = ctrl.process_events() {
                    eprintln!("Error processing events: {:?}", e);
                }
            }
            glib::ControlFlow::Continue
        });

        // The GTK main loop will handle the rest
    }
}
