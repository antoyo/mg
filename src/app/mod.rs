/*
 * Copyright (c) 2016-2017 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

//pub mod dialog;
mod message;
pub mod settings;
mod shortcut;
pub mod status_bar;

use std::borrow::Cow;
use std::cell::RefCell;
use std::char;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::mem;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gdk;
use gdk::enums::key::Key as GdkKey;
use gdk::{EventKey, RGBA};
use gdk::enums::key::{Escape, colon};
use glib;
use glib::translate::ToGlib;
use gtk;
use gtk::{
    ContainerExt,
    EditableSignals,
    Grid,
    Inhibit,
    IsA,
    OrientableExt,
    Overlay,
    PackType,
    Settings,
    WidgetExt,
    Window,
    WindowExt,
    WindowType,
    STATE_FLAG_NORMAL,
};
use gtk::prelude::WidgetExtManual;
use gtk::Orientation::Vertical;
use mg_settings::{self, Config, EnumFromStr, EnumMetaData, Parser, SettingCompletion};
use mg_settings::Command::{self, App, Custom, Map, Set, Unmap};
use mg_settings::error::{Error, Result};
use mg_settings::error::ErrorType::{MissingArgument, NoCommand, Parse, UnknownCommand};
use mg_settings::key::Key;
use relm::Widget;
use relm::gtk_ext::BoxExtManual;
use relm_attributes::widget;

use app::shortcut::shortcut_to_string;
use completion::{
    CommandCompleter,
    Completer,
    CompletionView,
    SettingCompleter,
    DEFAULT_COMPLETER_IDENT,
    NO_COMPLETER_IDENT,
};
use gobject::ObjectExtManual;
use self::ActivationType::{Current, Final};
use self::settings::NoSettings;
use self::ShortcutCommand::{Complete, Incomplete};
use self::status_bar::StatusBar;
use self::status_bar::Msg::*;
use self::Msg::*;
pub use self::status_bar::StatusBarItem;
use style_context::StyleContextExtManual;
use super::{Modes, NoSpecialCommands, SpecialCommand, Variables};

#[derive(PartialEq)]
enum ActivationType {
    Current,
    Final,
}

type Mappings = HashMap<&'static str, HashMap<Vec<Key>, String>>;
type ModesHash = HashMap<&'static str, &'static str>;

//type Modes = HashMap<String, String>;

/// A command from a map command.
#[derive(Debug)]
enum ShortcutCommand {
    /// A complete command that is to be executed.
    Complete(String),
    /// An incomplete command where the user needs to complete it and press Enter.
    Incomplete(String),
}

const TRANSPARENT: &RGBA = &RGBA { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0 };

const BLOCKING_INPUT_MODE: &str = "blocking-input";
const COMMAND_MODE: &str = "command";
const COMPLETE_NEXT_COMMAND: &str = "complete-next";
const COMPLETE_PREVIOUS_COMMAND: &str = "complete-previous";
const INPUT_MODE: &str = "input";
const NORMAL_MODE: &str = "normal";

#[derive(ManualClone)]
pub struct Model<COMM> {
    close_callback: Option<Rc<Fn() + Send + Sync>>,
    current_command_mode: char,
    current_mode: String,
    current_shortcut: Vec<Key>,
    entry_shown: bool,
    foreground_color: RGBA,
    mappings: Mappings,
    modes: ModesHash,
    settings_parser: Rc<RefCell<Parser<COMM>>>,
    //special_command_callback: Option<Box<Fn(SPEC)>>,
    variables: HashMap<String, fn() -> String>,
}

#[derive(Msg)]
pub enum Msg<COMM> {
    CustomCommand(COMM),
    EnterCommandMode,
    EnterNormalMode,
    KeyPress(GdkKey),
    KeyRelease(GdkKey),
    Quit,
}

#[widget]
impl<COMM: Clone + EnumFromStr + Send + 'static> Widget for Mg<COMM> {
    fn init_view(&self, model: &mut Model<COMM>) {
        // FIXME: this is executed too late, so the color is not good.
        model.foreground_color = self.get_foreground_color();
    }

    // TODO: switch from a &'static str to a String.
    fn model((user_modes, variables, settings_filename): (Modes, Variables, &'static str)) -> Model<COMM> {
        // TODO: show the error instead of unwrapping.
        let (parser, mappings, modes) = parse_config(settings_filename, user_modes, None).unwrap();
        Model {
            close_callback: None,
            current_command_mode: ':',
            current_mode: NORMAL_MODE.to_string(),
            current_shortcut: vec![],
            entry_shown: false,
            foreground_color: unsafe { mem::zeroed() },
            mappings: mappings,
            modes: modes,
            settings_parser: Rc::new(RefCell::new(parser)),
            //special_command_callback: None,
            variables: variables.iter().map(|&(string, func)| (string.to_string(), func)).collect(),
        }
    }

    fn update(&mut self, event: Msg<COMM>, model: &mut Self::Model) {
        match event {
            CustomCommand(_) => (), // To be listened to by the user.
            EnterCommandMode => {
                //self.status_bar.set_completer(DEFAULT_COMPLETER_IDENT);
                self.set_current_identifier(':', model);
                self.set_mode(COMMAND_MODE, model);
                self.reset(model);
                self.clear_shortcut(model);
                //self.status_bar.show_completion();
                self.show_entry(model);
            },
            EnterNormalMode => {
                let status_bar = self.status_bar.widget();
                model.entry_shown = false;
                //self.status_bar.hide_completion();
                self.set_mode(NORMAL_MODE, model);
                self.set_current_identifier(':', model);
                self.reset(model);
                self.clear_shortcut(model);
                /*if let Some(ref callback) = self.input_callback {
                    callback(None);
                }
                self.input_callback = None;*/
            },
            KeyPress(_) => println!("press"),
            KeyRelease(_) => println!("release"),
            Quit => {
                if let Some(ref callback) = model.close_callback {
                    callback();
                }
                else {
                    gtk::main_quit();
                }
            },
        }
    }

    view! {
        #[name="window"]
        gtk::Window {
            #[container]
            gtk::Box {
                orientation: Vertical,
                #[container="status-bar-item"]
                #[name="status_bar"]
                StatusBar {
                    Activate(input) with model => self.command_activate(input, model),
                    entry_shown: model.entry_shown,
                    packing: {
                        pack_type: PackType::End,
                    },
                    #[name="message"]
                    StatusBarItem {
                        text: "", // TODO: bind to a model field.
                    },
                    #[name="mode"]
                    StatusBarItem {
                        text: &model.current_mode,
                    },
                    #[name="shortcut"]
                    StatusBarItem {
                        text: &shortcut_to_string(&model.current_shortcut),
                    },
                },
            },
            key_press_event(_, key) with model => return self.key_press(&key, model),
            key_release_event(_, key) => (KeyRelease(key.get_keyval()), Inhibit(false)),
            delete_event(_, _) => (Quit, Inhibit(false)),
        }
    }
}

impl<COMM: EnumFromStr> Mg<COMM> {
    /// Convert an action String to a command String.
    fn action_to_command(&self, action: &str) -> ShortcutCommand {
        if let Some(':') = action.chars().next() {
            if let Some(index) = action.find("<Enter>") {
                Complete(action[1..index].to_string())
            }
            else {
                Incomplete(action[1..].to_string())
            }
        }
        else {
            Complete(action.to_string())
        }
    }

    /// Handle an application command.
    fn app_command(&self, command: &str) {
        match command {
            //COMPLETE_NEXT_COMMAND => self.status_bar.complete_next(),
            //COMPLETE_PREVIOUS_COMMAND => self.status_bar.complete_previous(),
            _ => unreachable!(),
        }
    }

    /// Call the callback with the command or show an error if the command cannot be parsed.
    fn call_command(&self, command: Result<Command<COMM>>) -> Option<Msg<COMM>> {
        match command {
            Ok(command) => {
                match command {
                    App(command) => self.app_command(&command),
                    Custom(command) => return Some(CustomCommand(command)),
                    Set(name, value) => {
                        /*match Sett::to_variant(&name, value) {
                            Ok(setting) => self.set_setting(setting),
                            Err(error) => {
                                let message = format!("Error setting value: {}", error);
                                self.error(&message);
                            },
                        }*/
                    },
                    _ => unimplemented!(),
                }
            },
            Err(error) => {
                if let Error::Parse(error) = error {
                    let message =
                        match error.typ {
                            MissingArgument => "Argument required".to_string(),
                            NoCommand => return None,
                            Parse => format!("Parse error: unexpected {}, expecting: {}", error.unexpected, error.expected),
                            UnknownCommand => format!("Not a command: {}", error.unexpected),
                        };
                    self.error(&message);
                }
            },
        }
        None
    }

    /// Handle the command entry activate event.
    fn command_activate(&self, input: Option<String>, model: &mut Model<COMM>) -> Msg<COMM> {
        let message =
        /*if self.current_mode == INPUT_MODE || self.current_mode == BLOCKING_INPUT_MODE {
            let mut should_reset = false;
            if let Some(ref callback) = self.input_callback {
                self.answer = input.clone();
                callback(input);
                should_reset = true;
            }
            if should_reset {
                self.reset();
            }
            self.input_callback = None;
            self.choices.clear();
        }
        else {*/
            self.handle_command(input, model)
        //}
        ;
        //self.return_to_normal_mode();
        message.unwrap()
    }

    /// Handle the key press event for the command mode.
    #[allow(non_upper_case_globals)]
    fn command_key_press(&self, key: &EventKey, model: &mut Model<COMM>) -> (Option<Msg<COMM>>, Inhibit) {
        match key.get_keyval() {
            Escape => {
                println!("Escape");
                // TODO: this should not call the callback (in update(EnterNormalMode)).
                (Some(EnterNormalMode), Inhibit(false))
            },
            _ => self.handle_shortcut(key, model),
        }
    }

    /// Connect the close event to the specified callback.
    pub fn connect_close<F: Fn() + Send + Sync + 'static>(&mut self, callback: F, model: &mut Model<COMM>) {
        model.close_callback = Some(Rc::new(callback));
    }

    /// Get the color of the text.
    fn get_foreground_color(&self) -> RGBA {
        let style_context = self.window.get_style_context().unwrap();
        style_context.get_color(STATE_FLAG_NORMAL)
    }

    /// Handle the command activate event.
    fn handle_command(&self, command: Option<String>, model: &mut Model<COMM>) -> Option<Msg<COMM>> {
        if let Some(command) = command {
            if model.current_command_mode == ':' {
                let result = (*model.settings_parser.borrow_mut()).parse_line(&command);
                return self.call_command(result);
            }
            else {
                self.handle_special_command(Final, &command, model);
            }
        }
        None
    }

    /// Handle a special command activate or key press event.
    fn handle_special_command(&self, activation_type: ActivationType, command: &str, model: &mut Model<COMM>) {
        let mut update_identifier = false;
        /*if let Some(ref callback) = model.special_command_callback {
            if let Ok(special_command) = Spec::identifier_to_command(model.current_command_mode, command) {
                callback(special_command);
                if activation_type == Final {
                    update_identifier = true;
                }
            }
        }*/
        if update_identifier {
            self.set_current_identifier(':', model);
        }
    }

    /// Input the specified command.
    fn input_command(&self, command: &str, model: &mut Model<COMM>) {
        self.set_mode(COMMAND_MODE, model);
        self.show_entry(model);
        let mut command = command.to_string();
        for (variable, function) in &model.variables {
            command = command.replace(&format!("<{}>", variable), &function());
        }
        let text: Cow<str> =
            if command.contains(' ') {
                command.into()
            }
            else {
                format!("{} ", command).into()
            };
        self.status_bar.widget().set_input(&text);
    }

    /// Handle the key press event for the input mode.
    #[allow(non_upper_case_globals)]
    fn input_key_press(&self, key: &EventKey, model: &mut Model<COMM>) -> (Option<Msg<COMM>>, Inhibit) {
        match key.get_keyval() {
            Escape => {
                (Some(EnterNormalMode), Inhibit(false))
            },
            keyval => {
                /*if self.handle_input_shortcut(key) {
                    return (None, Inhibit(true));
                }
                else if let Some(character) = char::from_u32(keyval) {
                    if self.choices.contains(&character) {
                        self.set_dialog_answer(&character.to_string());
                        return (None, Inhibit(true));
                    }
                }*/
                self.handle_shortcut(key, model)
            },
        }
    }

    /// Handle the key press event.
    fn key_press(&self, key: &EventKey, model: &mut Model<COMM>) -> (Option<Msg<COMM>>, Inhibit) {
        match model.current_mode.as_ref() {
            NORMAL_MODE => self.normal_key_press(key, model),
            COMMAND_MODE => self.command_key_press(key, model),
            BLOCKING_INPUT_MODE | INPUT_MODE => self.input_key_press(key, model),
            _ => self.handle_shortcut(key, model)
        }
    }

    /// Handle the key press event for the normal mode.
    #[allow(non_upper_case_globals)]
    fn normal_key_press(&self, key: &EventKey, model: &mut Model<COMM>) -> (Option<Msg<COMM>>, Inhibit) {
        match key.get_keyval() {
            colon => (Some(EnterCommandMode), Inhibit(true)),
            Escape => {
                self.reset(model);
                self.clear_shortcut(model);
                self.handle_shortcut(key, model)
            },
            keyval => {
                let character = keyval as u8 as char;
                /*if Spec::is_identifier(character) {
                    //self.status_bar.set_completer(NO_COMPLETER_IDENT);
                    self.set_current_identifier(character);
                    self.set_mode(COMMAND_MODE);
                    self.reset(model);
                    self.clear_shortcut(model);
                    self.show_entry(model);
                    Inhibit(true)
                }
                else {*/
                    self.handle_shortcut(key, model)
                //}
            },
        }
    }

    /// Handle the escape event.
    fn reset(&self, model: &mut Model<COMM>) {
        self.reset_colors(model);
        //self.status_bar.hide_widgets();
        //self.message.set_text("");
        self.show_mode(model);
        self.clear_shortcut(model);
    }

    /// Reset the background and foreground colors of the status bar.
    fn reset_colors(&self, model: &Model<COMM>) {
        let status_bar = self.status_bar.widget().root();
        // TODO: switch to CSS.
        status_bar.override_background_color(STATE_FLAG_NORMAL, TRANSPARENT);
        status_bar.override_color(STATE_FLAG_NORMAL, &model.foreground_color);
    }

    /// Set the current (special) command identifier.
    fn set_current_identifier(&self, identifier: char, model: &mut Model<COMM>) {
        model.current_command_mode = identifier;
        // TODO: the following should not be necessary when custom methods in the attribute will be
        // supported.
        self.status_bar.widget().set_identifier(&identifier.to_string());
    }

    /// Use the dark variant of the theme if available.
    pub fn set_dark_theme(&self, use_dark: bool) {
        println!("set_dark_theme");
        let settings = Settings::get_default().unwrap();
        settings.set_data("gtk-application-prefer-dark-theme", use_dark.to_glib());
        //self.foreground_color = Application::<Comm, Sett, Spec>::get_foreground_color(&self.window);
    }

    /// Set the current mode.
    pub fn set_mode(&self, mode: &str, model: &mut Model<COMM>) {
        model.current_mode = mode.to_string();
        self.show_mode(model);
        /*if let Some(ref callback) = self.mode_changed_callback {
            callback(mode);
        }*/
    }

    pub fn set_settings<P: AsRef<Path>>(&self, filename: P) {
    }

    /// Set the window title.
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    fn show_entry(&self, model: &mut Model<COMM>) {
        model.entry_shown = true;
        let status_bar = self.status_bar.widget();
        status_bar.set_entry_shown(true);
        status_bar.show_entry();
    }

    /// Show the current mode if it is not the normal mode.
    fn show_mode(&self, model: &Model<COMM>) {
        let label = self.mode.widget().root();
        if model.current_mode != NORMAL_MODE && model.current_mode != COMMAND_MODE && model.current_mode != INPUT_MODE
            && model.current_mode != BLOCKING_INPUT_MODE
        {
            label.set_text(&model.current_mode);
        }
        else {
            label.set_text("");
        }
    }
}

/// Parse a configuration file.
pub fn parse_config<P: AsRef<Path>, COMM: EnumFromStr>(filename: P, user_modes: Modes, include_path: Option<&str>)
    -> Result<(Parser<COMM>, Mappings, ModesHash)>
{
    let mut mappings = HashMap::new();
    let file = File::open(filename)?;
    let buf_reader = BufReader::new(file);

    let mut modes = HashMap::new();
    for &(key, value) in user_modes {
        modes.insert(key, value);
    }
    assert!(modes.insert("n", NORMAL_MODE).is_none(), "Duplicate mode prefix n.");
    assert!(modes.insert("c", COMMAND_MODE).is_none(), "Duplicate mode prefix c.");
    let config = Config {
        application_commands: vec![COMPLETE_NEXT_COMMAND.to_string(), COMPLETE_PREVIOUS_COMMAND.to_string()],
        mapping_modes: modes.keys().cloned().collect(),
    };
    let mut parser = Parser::new_with_config(config);
    if let Some(include_path) = include_path {
        parser.set_include_path(include_path);
    }

    let commands = parser.parse(buf_reader)?;
    for command in commands {
        match command {
            //App(command) => self.app_command(&command),
            //Custom(command) => {
            //if let Some(ref callback) = self.command_callback {
            //callback(command);
            //}
            //},
            Map { action, keys, mode } => {
                let mode_mappings = mappings.entry(modes[mode.as_str()].clone()).or_insert_with(HashMap::new);
                mode_mappings.insert(keys, action);
            },
            _ => (), // TODO: to remove.
            //Set(name, value) => self.set_setting(Sett::to_variant(&name, value)?),
            //Unmap { .. } => panic!("not yet implemented"), // TODO
        }
    }
    Ok((parser, mappings, modes))
}

/*
/// Alias for an application builder without settings.
pub type SimpleApplicationBuilder = ApplicationBuilder<NoSettings>;

/// Application builder.
pub struct ApplicationBuilder<Sett: mg_settings::settings::Settings> {
    completers: HashMap<String, Box<Completer>>,
    modes: Option<Modes>,
    include_path: Option<PathBuf>,
    settings: Option<Sett>,
}

impl<Sett: mg_settings::settings::Settings + 'static> ApplicationBuilder<Sett> {
    /// Create a new application builder.
    #[allow(new_without_default_derive)]
    pub fn new() -> Self {
        ApplicationBuilder {
            completers: HashMap::new(),
            modes: None,
            include_path: None,
            settings: None,
        }
    }

    /// Create a new application with configuration and include path.
    pub fn build<Spec, Comm>(self) -> Box<Application<Comm, Sett, Spec>>
        where Spec: SpecialCommand + 'static,
              Comm: EnumFromStr + EnumMetaData + 'static,
              Sett: EnumMetaData + SettingCompletion,
    {
        Application::new(self)
    }

    /// Add a input completer.
    pub fn completer<C: Completer + 'static>(mut self, name: &str, completer: C) -> Self {
        self.completers.insert(name.to_string(), Box::new(completer));
        self
    }

    /// Set the include path of the configuration files.
    pub fn include_path<P: AsRef<Path>>(mut self, include_path: P) -> Self {
        self.include_path = Some(include_path.as_ref().to_path_buf());
        self
    }

    /// Set the configuration of the application.
    pub fn modes(mut self, mut modes: Modes) -> Self {
        assert!(modes.insert("n".to_string(), NORMAL_MODE.to_string()).is_none(), "Duplicate mode prefix n.");
        assert!(modes.insert("c".to_string(), COMMAND_MODE.to_string()).is_none(), "Duplicate mode prefix c.");
        self.modes = Some(modes);
        self
    }

    /// Set the default settings of the application.
    pub fn settings(mut self, settings: Sett) -> Self {
        self.settings = Some(settings);
        self
    }
}
*/

/*
/// Create a new MG application window.
/// This window contains a status bar where the user can type a command and a central widget.
pub struct Application<Comm, Sett: mg_settings::settings::Settings = NoSettings, Spec = NoSpecialCommands> {
    answer: Option<String>,
    command_callback: Option<Box<Fn(Comm)>>,
    choices: Vec<char>,
    close_callback: Option<Box<Fn()>>,
    current_command_mode: char,
    current_mode: String,
    current_shortcut: Vec<Key>,
    foreground_color: RGBA,
    input_callback: Option<Box<Fn(Option<String>)>>,
    mappings: HashMap<String, HashMap<Vec<Key>, String>>,
    modes: Modes,
    message: StatusBarItem,
    mode_changed_callback: Option<Box<Fn(&str)>>,
    mode_label: StatusBarItem,
    settings: Option<Sett>,
    settings_parser: Parser<Comm>,
    setting_change_callback: Option<Box<Fn(&Sett::Variant)>>,
    shortcuts: HashMap<Key, String>,
    shortcut_label: StatusBarItem,
    shortcut_pressed: bool,
    special_command_callback: Option<Box<Fn(Spec)>>,
    status_bar: Box<StatusBar>,
    view: Overlay,
    variables: HashMap<String, Box<Fn() -> String>>,
    window: Window,
}

impl<Spec, Comm, Sett> Application<Comm, Sett, Spec>
    where Spec: SpecialCommand + 'static,
          Comm: EnumFromStr + EnumMetaData + 'static,
          Sett: mg_settings::settings::Settings + EnumMetaData + SettingCompletion + 'static,
{
    fn new(builder: ApplicationBuilder<Sett>) -> Box<Self> {
        let modes = builder.modes.unwrap_or_default();
        let config = Config {
            application_commands: vec![COMPLETE_NEXT_COMMAND.to_string(), COMPLETE_PREVIOUS_COMMAND.to_string()],
            mapping_modes: modes.keys().cloned().collect(),
        };

        let view = Overlay::new();
        grid.attach(&view, 0, 0, 1, 1);

        let completion_view = CompletionView::new();
        view.add_overlay(&**completion_view);

        let mut completers: HashMap<String, Box<Completer>> = HashMap::new();
        completers.insert(DEFAULT_COMPLETER_IDENT.to_string(), Box::new(CommandCompleter::<Comm>::new()));
        completers.insert("set".to_string(), Box::new(SettingCompleter::<Sett>::new()));
        for (identifier, completer) in builder.completers {
            completers.insert(identifier, completer);
        }
        let mut status_bar = StatusBar::new(completion_view, completers);
        grid.attach(&**status_bar, 0, 2, 1, 1);
        window.show_all();
        status_bar.hide_widgets();
        status_bar.hide_completion();

        let foreground_color = Application::<Comm, Sett, Spec>::get_foreground_color(&window);

        let mut parser = Parser::new_with_config(config);
        if let Some(include_path) = builder.include_path {
            parser.set_include_path(include_path);
        }

        let mut app = Box::new(Application {
            answer: None,
            command_callback: None,
            choices: vec![],
            close_callback: None,
            current_command_mode: ':',
            current_mode: current_mode,
            current_shortcut: vec![],
            foreground_color: foreground_color,
            input_callback: None,
            mappings: HashMap::new(),
            modes: modes,
            message: StatusBarItem::new().left(),
            mode_changed_callback: None,
            mode_label: StatusBarItem::new().left(),
            settings: builder.settings,
            settings_parser: parser,
            setting_change_callback: None,
            shortcuts: HashMap::new(),
            shortcut_label: StatusBarItem::new(),
            shortcut_pressed: false,
            special_command_callback: None,
            //status_bar: status_bar,
            view: view,
            variables: HashMap::new(),
            window: window,
        });

        //app.status_bar.add_item(&app.shortcut_label);
        //app.status_bar.add_item(&app.mode_label);
        //app.status_bar.add_item(&app.message);

        //connect!(app.window, connect_delete_event(_, _), app, quit);
        //connect!(app.status_bar, connect_activate(input), app, command_activate(input));
        //connect!(app.window, connect_key_press_event(_, key), app, key_press(key));
        //connect!(app.window, connect_key_release_event(_, key), app, key_release(key));
        //connect!(app.status_bar.entry, connect_changed(_), app, update_completions);

        app
    }

    /// Create a new status bar item.
    pub fn add_statusbar_item(&self) -> StatusBarItem {
        let item = StatusBarItem::new();
        //self.status_bar.add_item(&item);
        item
    }

    /// Add a variable that can be used in mappings.
    /// The placeholder will be replaced by the value return by the function.
    pub fn add_variable<F: Fn() -> String + 'static>(&mut self, variable_name: &str, function: F) {
        self.variables.insert(variable_name.to_string(), Box::new(function));
    }

    /// Call the setting changed callback.
    fn call_setting_callback(&self, setting: &Sett::Variant) {
        if let Some(ref callback) = self.setting_change_callback {
            callback(setting);
        }
    }

    /// Handle the key release event for the command mode.
    fn command_key_release(&mut self, _key: &EventKey) -> Inhibit {
        /*if self.current_command_mode != ':' && Spec::is_always(self.current_command_mode) {
            if let Some(command) = self.status_bar.get_command() {
                self.handle_special_command(Current, &command);
            }
        }*/
        Inhibit(false)
    }

    /// Connect the close event to the specified callback.
    pub fn connect_close<F: Fn() + 'static>(&mut self, callback: F) {
        self.close_callback = Some(Box::new(callback));
    }

    /// Add a callback to the command event.
    pub fn connect_command<F: Fn(Comm) + 'static>(&mut self, callback: F) {
        self.command_callback = Some(Box::new(callback));
    }

    /// Add a callback to the window key press event.
    pub fn connect_key_press_event<F: Fn(&Window, &EventKey) -> Inhibit + 'static>(&self, callback: F) {
        self.window.connect_key_press_event(callback);
    }

    /// Add a callback to change mode event.
    pub fn connect_mode_changed<F: Fn(&str) + 'static>(&mut self, callback: F) {
        self.mode_changed_callback = Some(Box::new(callback));
    }

    /// Add a callback to setting changed event.
    pub fn connect_setting_changed<F: Fn(&Sett::Variant) + 'static>(&mut self, callback: F) {
        self.setting_change_callback = Some(Box::new(callback));
    }

    /// Add a callback to the special command event.
    pub fn connect_special_command<F: Fn(Spec) + 'static>(&mut self, callback: F) {
        self.special_command_callback = Some(Box::new(callback));
    }

    /// Delete the current completion item.
    pub fn delete_current_completion_item(&self) {
        //self.status_bar.delete_current_completion_item();
    }

    /// Get the text of the status bar command entry.
    pub fn get_command(&self) -> String {
        String::new() // TODO: remove
        //self.status_bar.get_command().unwrap_or_default()
    }

    /// Get the current mode.
    pub fn get_mode(&self) -> &str {
        &self.current_mode
    }

    /// Handle the key release event.
    fn key_release(&mut self, key: &EventKey) -> Inhibit {
        match self.current_mode.as_ref() {
            COMMAND_MODE => self.command_key_release(key),
            _ => Inhibit(false),
        }
    }

    /// Call the callback added by the user.
    /// Otherwise, exit the main loop.
    fn quit(&self) -> Inhibit {
        if let Some(ref callback) = self.close_callback {
            callback();
        }
        else {
            gtk::main_quit();
        }
        Inhibit(true)
    }

    /// Get the settings.
    pub fn settings(&self) -> &Sett {
        self.settings.as_ref().unwrap()
    }

    /// Set a setting value.
    pub fn set_setting(&mut self, setting: Sett::Variant) {
        self.call_setting_callback(&setting);
        if let Some(ref mut settings) = self.settings {
            settings.set_value(setting);
        }
    }

    /// Set the answer to return to the caller of the dialog.
    fn set_dialog_answer(&mut self, answer: &str) {
        let mut should_reset = false;
        if self.current_mode == BLOCKING_INPUT_MODE {
            self.answer = Some(answer.to_string());
            gtk::main_quit();
        }
        else if let Some(ref callback) = self.input_callback {
            callback(Some(answer.to_string()));
            self.choices.clear();
            should_reset = true;
        }
        if should_reset {
            self.return_to_normal_mode();
            self.reset();
        }
        self.input_callback = None;
    }

    /// Set the main widget.
    pub fn set_view<W: IsA<gtk::Widget> + WidgetExt>(&self, view: &W) {
        view.set_hexpand(true);
        view.set_vexpand(true);
        view.show_all();
        self.view.add(view);
    }

    /// Set the window title.
    pub fn set_window_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Update the completions of the status bar.
    fn update_completions(&mut self) {
        //self.status_bar.update_completions(&self.current_mode);
    }

    /// Get the application window.
    pub fn window(&self) -> &Window {
        &self.window
    }
}*/
