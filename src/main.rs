use gtk::prelude::*;
use gtk::Application;
use waycast::ui::AppModel;

fn main() {
    let app = Application::builder()
        .application_id("dev.thegrind.waycast")
        .build();

    app.connect_activate(|app| {
        let model = AppModel::new(app);
        model.borrow().show();
    });

    app.run();
}