use crate::edit_view::EditView;
use crate::main_win::MainState;
use crate::pref_storage::*;
use crate::rpc::Core;
use gettextrs::gettext;
use gio::{ActionMapExt, SimpleAction};
use gtk::*;
use log::{debug, error, trace};
use pango::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PrefsWin {
    core: Rc<RefCell<Core>>,
    window: Window,
}

impl PrefsWin {
    pub fn new(
        parent: &ApplicationWindow,
        main_state: &Rc<RefCell<MainState>>,
        core: &Rc<RefCell<Core>>,
        edit_view: Option<Rc<RefCell<EditView>>>,
    ) -> Rc<RefCell<Self>> {
        const SRC: &str = include_str!("ui/prefs_win.glade");
        let builder = Builder::new_from_string(SRC);
        let application = parent.get_application().unwrap();

        let window: Window = builder.get_object("prefs_win").unwrap();
        let font_chooser_widget: FontChooserWidget =
            builder.get_object("font_chooser_widget").unwrap();
        let theme_combo_box: ComboBoxText = builder.get_object("theme_combo_box").unwrap();
        let margin_spinbutton: SpinButton = builder.get_object("margin_spinbutton").unwrap();
        let tab_size_spinbutton: SpinButton = builder.get_object("tab_size_spinbutton").unwrap();

        let xi_config = &main_state.borrow().config;

        {
            let mut font_desc = FontDescription::new();
            let font_face = &xi_config.config.font_face;
            font_desc.set_size(xi_config.config.font_size as i32 * pango::SCALE);
            font_desc.set_family(font_face);

            trace!("{}: {}", gettext("Setting font description"), font_face);

            font_chooser_widget.set_font_desc(&font_desc);
        }

        {
            font_chooser_widget.connect_property_font_desc_notify(
                clone!(main_state => move |font_widget| {
                    if let Some(font_desc) = font_widget.get_font_desc() {
                        let mut font_conf = &mut main_state.borrow_mut().config;

                        let font_family = font_desc.get_family().unwrap();
                        let font_size = font_desc.get_size() / pango::SCALE;
                        debug!("{} {}", gettext("Setting font to"), &font_family);
                        debug!("{} {}", gettext("Setting font size to"), &font_size);

                        font_conf.config.font_size = font_size as u32;
                        font_conf.config.font_face = font_family.to_string();
                        font_conf
                            .save()
                            .map_err(|e| error!("{}", e.to_string()))
                            .unwrap();
                    }
                }),
            );
        }

        {
            let main_state = main_state.borrow();
            for (i, theme_name) in main_state.themes.iter().enumerate() {
                theme_combo_box.append_text(theme_name);
                if &main_state.theme_name == theme_name {
                    trace!("{}: {}", gettext("Setting active theme"), i);
                    theme_combo_box.set_active(Some(i as u32));
                }
            }
        }

        theme_combo_box.connect_changed(clone!(core, main_state => move |cb|{
            if let Some(theme_name) = cb.get_active_text() {
                debug!("{} {:?}", gettext("Theme changed to"), &theme_name);
                let core = core.borrow();
                core.set_theme(&theme_name);

                crate::pref_storage::set_theme_schema(theme_name.to_string());

                let mut main_state = main_state.borrow_mut();
                main_state.theme_name = theme_name.to_string();
            }
        }));

        {
            let scroll_past_end_checkbutton = SimpleAction::new_stateful(
                "scroll_past_end",
                None,
                &main_state
                    .borrow()
                    .config
                    .config
                    .scroll_past_end
                    .to_variant(),
            );

            scroll_past_end_checkbutton.connect_change_state(
                clone!(main_state => move |action, value| {
                    if let Some(value) = value.as_ref() {
                        action.set_state(value);
                        let value: bool = value.get().unwrap();
                        debug!("{}: {}", gettext("Scrolling past end"), value);
                        main_state.borrow_mut().config.config.scroll_past_end = value;
                        main_state.borrow().config.save()
                            .map_err(|e| error!("{}", e.to_string()))
                            .unwrap();
                    }
                }),
            );
            application.add_action(&scroll_past_end_checkbutton);
            scroll_past_end_checkbutton.set_enabled(true);
        }

        {
            let word_wrap_checkbutton = SimpleAction::new_stateful(
                "word_wrap",
                None,
                &main_state.borrow().config.config.word_wrap.to_variant(),
            );

            word_wrap_checkbutton.connect_change_state(clone!(main_state => move |action,value| {
                if let Some(value) = value.as_ref() {
                    action.set_state(value);
                    let value: bool = value.get().unwrap();
                    debug!("{}: {}", gettext("Word wrapping"), value);
                    main_state.borrow_mut().config.config.word_wrap = value;
                    main_state.borrow().config.save()
                        .map_err(|e| error!("{}", e.to_string()))
                        .unwrap();
                }
            }));
            application.add_action(&word_wrap_checkbutton);
        }

        {
            let tab_stops_checkbutton = SimpleAction::new_stateful(
                "tap_stops",
                None,
                &main_state.borrow().config.config.use_tab_stops.to_variant(),
            );

            tab_stops_checkbutton.connect_change_state(clone!(main_state => move |action,value| {
                if let Some(value) = value.as_ref() {
                    action.set_state(value);
                    let value: bool = value.get().unwrap();
                    debug!("{}: {}", gettext("Tab stops"), value);
                    main_state.borrow_mut().config.config.use_tab_stops = value;
                    main_state.borrow().config.save()
                        .map_err(|e| error!("{}", e.to_string()))
                        .unwrap();
                }
            }));
            application.add_action(&tab_stops_checkbutton);
        }

        {
            let draw_trailing_spaces_checkbutton = SimpleAction::new_stateful(
                "draw_trailing_spaces",
                None,
                &get_draw_trailing_spaces_schema().to_variant(),
            );

            draw_trailing_spaces_checkbutton.connect_change_state(move |action, value| {
                if let Some(value) = value.as_ref() {
                    action.set_state(value);
                    let value: bool = value.get().unwrap();
                    set_draw_trailing_spaces_schema(value);
                }
            });
            application.add_action(&draw_trailing_spaces_checkbutton);
        }

        {
            let margin_checkbutton = SimpleAction::new_stateful(
                "display_margin",
                None,
                &get_draw_right_margin().to_variant(),
            );

            margin_checkbutton.connect_change_state(
                clone!(edit_view, margin_spinbutton => move |action, value| {
                     if let Some(value) = value.as_ref() {
                        action.set_state(value);
                        let value: bool = value.get().unwrap();
                        debug!("{}: {}", gettext("Right hand margin"), value);
                        set_draw_right_margin(value);
                        if let Some(ev) = edit_view.clone() {
                            ev.borrow().view_item.edit_area.queue_draw();
                        }
                        margin_spinbutton.set_sensitive(value);
                    }
                }),
            );
            application.add_action(&margin_checkbutton);
        }

        {
            margin_spinbutton.set_sensitive(get_draw_right_margin());
            margin_spinbutton.set_value(f64::from(get_column_right_margin()));

            margin_spinbutton.connect_value_changed(clone!(edit_view => move |spin_btn| {
                let value = spin_btn.get_value();
                debug!("{}: {}", gettext("Right hand margin width"), value);
                set_column_right_margin(value as u32);
                if let Some(ev) = edit_view.clone() {
                    ev.borrow().view_item.edit_area.queue_draw();
                }
            }));
        }

        {
            tab_size_spinbutton.set_value(f64::from(main_state.borrow().config.config.tab_size));

            tab_size_spinbutton.connect_value_changed(
                clone!(main_state, edit_view => move |spin_btn| {
                    let value = spin_btn.get_value();
                    debug!("{}: {}", gettext("Setting tab size"), value);
                    main_state.borrow_mut().config.config.tab_size = value as u32;
                    main_state.borrow().config.save()
                    .map_err(|e| error!("{}", e.to_string()))
                    .unwrap();
                    if let Some(ev) = edit_view.clone() {
                        ev.borrow().view_item.edit_area.queue_draw();
                    }
                }),
            );
        }

        {
            let highlight_line_checkbutton = SimpleAction::new_stateful(
                "highlight_line",
                None,
                &get_highlight_line().to_variant(),
            );

            highlight_line_checkbutton.connect_change_state(
                clone!(edit_view => move |action, value| {
                    if let Some(value) = value.as_ref() {
                        action.set_state(value);
                        let value: bool = value.get().unwrap();
                        set_highlight_line(value);
                        if let Some(ev) = edit_view.clone() {
                            ev.borrow().view_item.edit_area.queue_draw();
                        }
                     }
                }),
            );
            application.add_action(&highlight_line_checkbutton);
        }

        let prefs_win = Rc::new(RefCell::new(Self {
            core: core.clone(),
            window: window.clone(),
        }));

        window.set_transient_for(Some(parent));
        window.show_all();

        prefs_win
    }
}
