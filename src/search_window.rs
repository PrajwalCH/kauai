use std::cell::RefCell;
use std::rc::Rc;

use gtk::gio;
use gtk::glib::{self, clone};
use gtk::prelude::*;

use crate::search_results::SearchResults;

pub struct SearchWindow {
    window: gtk::ApplicationWindow,
    scrollable_container: gtk::ScrolledWindow,
    search_results: Rc<RefCell<SearchResults>>,
    installed_apps: Rc<Vec<gio::AppInfo>>,
}

impl SearchWindow {
    pub fn new(app: &gtk::Application) -> Self {
        let window = gtk::ApplicationWindow::new(app);
        window.set_height_request(50);
        window.set_width_request(500);
        window.set_resizable(false);

        let main_container = gtk::Box::new(gtk::Orientation::Vertical, 5);
        main_container.append(&Self::create_search_box_widget());

        let scrollable_container = gtk::ScrolledWindow::new();
        scrollable_container.set_min_content_height(200);
        // Only show it when we get the results later
        scrollable_container.set_visible(false);

        let search_results = SearchResults::new();
        scrollable_container.set_child(Some(search_results.container()));
        main_container.append(&scrollable_container);
        window.set_child(Some(&main_container));

        Self {
            window,
            scrollable_container,
            search_results: Rc::new(RefCell::new(search_results)),
            installed_apps: Rc::new(gio::AppInfo::all()),
        }
    }

    pub fn present(&self) {
        self.window.add_action(&self.create_search_action());
        self.window.present();
    }

    fn create_search_box_widget() -> gtk::SearchEntry {
        let search_box = gtk::SearchEntry::builder().hexpand(true).build();
        search_box.connect_search_changed(move |search_box| {
            let search_query = search_box.text().to_string();
            search_box
                .activate_action("win.search", Some(&search_query.to_variant()))
                .expect("search action should exist");
        });
        search_box
    }

    fn create_search_action(&self) -> gio::SimpleAction {
        let scrollable_container = &self.scrollable_container;
        let search_results = Rc::clone(&self.search_results);
        let installed_apps = Rc::clone(&self.installed_apps);
        let search_action = gio::SimpleAction::new("search", Some(&String::static_variant_type()));

        search_action.connect_activate(
            clone!(@weak scrollable_container => move |_state, variant| {
                if let Some(variant) = variant {
                    let mut search_results = search_results.borrow_mut();
                    // Clear previous results
                    search_results.clear();
                    scrollable_container.hide();

                    let search_query = variant.get::<String>().unwrap_or_default();
                    if search_query.is_empty() {
                        return;
                    }
                    let query_matched_apps = installed_apps.iter().filter_map(|app| {
                        let app_name = app.name();
                        if app_name.matches(&search_query).count() != 0 {
                            Some(app_name)
                        } else {
                            None
                        }
                    }).collect::<Vec<glib::GString>>();

                    if query_matched_apps.is_empty() {
                        return;
                    }

                    for app_name in &query_matched_apps {
                        search_results.push(gtk::Text::builder().text(app_name.as_str()).build());
                    }
                    scrollable_container.show();
                }
            }),
        );
        search_action
    }
}
