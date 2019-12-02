/*
 * Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com>
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

//! Non-modal input dialogs.

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use gtk;
use mg_settings::{EnumFromStr, EnumMetaData, SettingCompletion, SpecialCommand};
use mg_settings::key::Key;
use mg_settings::settings;
use relm::{
    ContainerComponent,
    EventStream,
    Relm,
    Update,
    Widget,
};

use app::{Mg, BLOCKING_INPUT_MODE, INPUT_MODE};
use app::color::color_blue;
use app::Msg::{
    BlockingCustomDialog,
    BlockingInput,
    BlockingQuestion,
    BlockingYesNoQuestion,
    EnterNormalModeAndReset,
    Input,
    Question,
    ResetInput,
    YesNoQuestion,
};
use app::status_bar::Msg::{Identifier, ShowIdentifier};
use completion::NO_COMPLETER_IDENT;
use completion::completion_view::Msg::SetOriginalInput;
use self::DialogResult::{Answer, Shortcut};

/// A Responder is a way to send back the answer of a dialog to the code that showed this dialog.
pub trait Responder {
    /// Send the answer back.
    /// It can be to a widget or a channel.
    fn respond(&self, answer: DialogResult);
}

/// Blocking input dialog responder.
/// This is used to send the message to a channel when the user answers the dialog.
pub struct BlockingInputDialog {
    tx: SyncSender<Option<String>>,
}

impl BlockingInputDialog {
    /// Create a new blocking input dialog responder.
    /// The answer will be sent to the receiver.
    pub fn new() -> (Self, Receiver<Option<String>>) {
        let (tx, rx) = sync_channel(1);
        (BlockingInputDialog {
            tx,
        }, rx)
    }
}

impl Responder for BlockingInputDialog {
    fn respond(&self, answer: DialogResult) {
        match answer {
            Answer(answer) => {
                self.tx.send(answer).expect("Sending answer shouldn't fail")
            },
            _ => unimplemented!(),
        }
        gtk::main_quit();
    }
}

/// Input dialog responder.
/// This is used to specify which message to send to which widget when the user answers the dialog.
pub struct InputDialog<WIDGET: Widget> {
    callback: Box<dyn Fn(Option<String>) -> WIDGET::Msg>,
    stream: EventStream<WIDGET::Msg>,
}

impl<WIDGET: Widget> InputDialog<WIDGET> {
    /// Create a new input dialog responder.
    /// The `callback` is a message constructor.
    /// The message will be sent to `relm` stream.
    pub fn new<F>(relm: &Relm<WIDGET>, callback: F) -> Self
        where F: Fn(Option<String>) -> WIDGET::Msg + 'static,
    {
        InputDialog {
            callback: Box::new(callback),
            stream: relm.stream().clone(),
        }
    }
}

impl<WIDGET: Widget> Responder for InputDialog<WIDGET> {
    fn respond(&self, answer: DialogResult) {
        match answer {
            Answer(answer) => self.stream.emit((self.callback)(answer)),
            _ => unimplemented!(),
        }
    }
}

/// Yes/no question input dialog responder.
/// This is used to specify which message to send to which widget when the user answers the dialog.
pub struct YesNoInputDialog<WIDGET: Widget> {
    callback: Box<dyn Fn(bool) -> WIDGET::Msg>,
    stream: EventStream<WIDGET::Msg>,
}

impl<WIDGET: Widget> YesNoInputDialog<WIDGET> {
    pub fn new<F>(relm: &Relm<WIDGET>, callback: F) -> Self
        where F: Fn(bool) -> WIDGET::Msg + 'static,
    {
        YesNoInputDialog {
            callback: Box::new(callback),
            stream: relm.stream().clone(),
        }
    }
}

impl<WIDGET: Widget> Responder for YesNoInputDialog<WIDGET> {
    fn respond(&self, answer: DialogResult) {
        match answer {
            Answer(answer) => self.stream.emit((self.callback)(answer == Some("y".to_string()))),
            _ => unimplemented!(),
        }
    }
}

/// Builder to create a new dialog.
pub struct DialogBuilder {
    /// Whether the dialog should block the calling function.
    blocking: bool,
    /// The available choices to the question.
    choices: Vec<char>,
    /// The text completer identifier for the input.
    completer: Option<String>,
    /// The default answer to the question.
    default_answer: String,
    /// The message/question to show to the user.
    message: String,
    /// The wrapper over the callback function to call for an asynchronous input dialog.
    responder: Option<Box<dyn Responder>>,
    /// The available shortcuts.
    shortcuts: HashMap<Key, String>,
}

impl DialogBuilder {
    /// Create a new dialog builder.
    #[allow(unknown_lints, new_without_default_derive)]
    pub fn new() -> Self {
        DialogBuilder {
            blocking: false,
            choices: vec![],
            completer: None,
            default_answer: String::new(),
            message: String::new(),
            responder: None,
            shortcuts: HashMap::new(),
        }
    }

    /// Set whether the dialog will block or not.
    pub fn blocking(mut self, blocking: bool) -> Self {
        self.blocking = blocking;
        self
    }

    /// Set the choices available for the input.
    pub fn choices(mut self, choices: Vec<char>) -> Self {
        self.choices = choices;
        self
    }

    /// Set the text completer for the input.
    pub fn completer(mut self, completer: &str) -> Self {
        self.completer = Some(completer.to_string());
        self
    }

    /// Set the default answer for the input.
    pub fn default_answer(mut self, answer: String) -> Self {
        self.default_answer = answer;
        self
    }

    /// Set the message/question to show to the user.
    pub fn message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    /// Set a responder for an asynchronous dialog.
    pub fn responder(mut self, responder: Box<dyn Responder>) -> Self {
        self.responder = Some(responder);
        self
    }

    /// Add a shortcut.
    pub fn shortcut(mut self, shortcut: Key, value: &str) -> Self {
        self.shortcuts.insert(shortcut, value.to_string());
        self
    }
}

/// Struct representing a dialog result.
/// A dialog result is either what the user typed in the input (Answer) or the string associated
/// with the shortcut.
pub enum DialogResult {
    /// A string typed by the user or None if the user closed the dialog (with Escape).
    Answer(Option<String>),
    /// A shortcut pressd by the user.
    Shortcut(String),
}

impl<COMM, SETT> Mg<COMM, SETT>
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
{
    /// Ask a question to the user and block until the user provides it (or cancel).
    pub fn blocking_custom_dialog(&mut self, responder: Box<dyn Responder>, builder: DialogBuilder) {
        let builder = builder
            .blocking(true)
            .responder(responder);
        self.show_dialog_without_shortcuts(builder);
    }

    /// Ask a question to the user and block until the user provides it (or cancel).
    pub fn blocking_input(&mut self, responder: Box<dyn Responder>, message: String, default_answer: String) {
        let builder = DialogBuilder::new()
            .blocking(true)
            .default_answer(default_answer)
            .message(message)
            .responder(responder);
        self.show_dialog_without_shortcuts(builder);
    }

    /// Ask a multiple-choice question to the user and block until the user provides it (or cancel).
    pub fn blocking_question(&mut self, responder: Box<dyn Responder>, message: String, choices: Vec<char>) {
        let builder = DialogBuilder::new()
            .blocking(true)
            .choices(choices)
            .message(message)
            .responder(responder);
        self.show_dialog_without_shortcuts(builder);
    }

    /// Show a blocking yes/no question.
    pub fn blocking_yes_no_question(&mut self, responder: Box<dyn Responder>, message: String) {
        let builder = DialogBuilder::new()
            .blocking(true)
            .choices(vec!['y', 'n'])
            .message(message)
            .responder(responder);
        self.show_dialog_without_shortcuts(builder);
    }

    /// Ask a question to the user.
    // TODO: use Option<String> for default_answer?
    pub fn input(&mut self, responder: Box<dyn Responder>, message: String, default_answer: String) {
        let builder = DialogBuilder::new()
            .default_answer(default_answer)
            .message(message)
            .responder(responder);
        self.show_dialog(builder);
    }

    /// Ask a multiple-choice question to the user.
    pub fn question(&mut self, responder: Box<dyn Responder>, message: String, choices: &[char]) {
        let builder = DialogBuilder::new()
            .responder(responder)
            .message(message)
            .choices(choices.to_vec());
        self.show_dialog(builder);
    }

    /// Set the answer to return to the caller of the dialog.
    pub fn set_dialog_answer(&mut self, answer: &str) {
        let mut should_reset = false;
        if let Some(callback) = self.model.input_callback.take() {
            callback(Some(answer.to_string()), self.model.shortcut_pressed);
            self.model.choices.clear();
            should_reset = true;
        }
        if should_reset {
            self.model.relm.stream().emit(EnterNormalModeAndReset);
        }
    }

    /// Show a dialog created with a `DialogBuilder`.
    pub fn show_dialog(&mut self, mut dialog_builder: DialogBuilder) {
        self.model.shortcut_pressed = false;

        self.model.shortcuts.clear();
        for (key, value) in dialog_builder.shortcuts {
            self.model.shortcuts.insert(key, value);
        }

        let choices = dialog_builder.choices.clone();
        if !choices.is_empty() {
            self.model.choices.clear();
            self.model.choices.append(&mut dialog_builder.choices);
            let choices: Vec<_> = choices.iter().map(|c| c.to_string()).collect();
            let choices = choices.join("/");
            self.status_bar.emit(Identifier(format!("{} ({}) ", dialog_builder.message, choices)));
            self.status_bar.emit(ShowIdentifier);
        }
        else {
            self.status_bar.emit(Identifier(format!("{} ", dialog_builder.message)));
            self.show_entry();
            self.set_input(&dialog_builder.default_answer);
        }

        if let Some(completer) = dialog_builder.completer {
            self.set_completer(&completer);
            self.model.completion_view.emit(SetOriginalInput(dialog_builder.default_answer));
            self.show_completion();
        }
        else {
            self.set_completer(NO_COMPLETER_IDENT);
        }

        self.model.answer = None;
        if dialog_builder.blocking {
            self.set_mode(BLOCKING_INPUT_MODE);
        }
        else {
            self.set_mode(INPUT_MODE);
        }
        if let Some(responder) = dialog_builder.responder {
            self.model.input_callback = Some(Box::new(move |answer, shortcut_pressed| {
                let answer =
                    if shortcut_pressed {
                        if let Some(answer) = answer.clone() {
                            Shortcut(answer)
                        }
                        else {
                            Answer(None)
                        }
                    }
                    else {
                        Answer(answer.clone())
                    };
                responder.respond(answer);
            }));
        }
        color_blue(self.status_bar.widget());
    }

    /// Show a dialog created with a `DialogBuilder` which does not contain shortcut.
    pub fn show_dialog_without_shortcuts(&mut self, dialog_builder: DialogBuilder) {
        self.show_dialog(dialog_builder);
            /*Answer(answer) => answer,
            Shortcut(_) => panic!("cannot return a shortcut in show_dialog_without_shortcuts()"),
        }*/
        // TODO
    }

    /// Show a yes/no question.
    pub fn yes_no_question(&mut self, responder: Box<dyn Responder>, message: String) {
        let builder = DialogBuilder::new()
            .choices(vec!['y', 'n'])
            .message(message)
            .responder(responder);
        self.show_dialog(builder);
    }
}

/// Ask a question to the user and block until the user provides it (or cancel).
pub fn blocking_dialog<COMM, SETT>(mg: &EventStream<<Mg<COMM, SETT> as Update>::Msg>, builder: DialogBuilder)
    -> Option<String>
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
{
    let (blocking_input_dialog, rx) = BlockingInputDialog::new();
    let responder = Box::new(blocking_input_dialog);
    mg.emit(BlockingCustomDialog(responder, builder));
    gtk::main();
    mg.emit(ResetInput);
    // TODO: really return None if no value was received?
    rx.try_recv().unwrap_or(None)
}

/// Ask a question to the user and block until the user provides it (or cancel).
pub fn blocking_input<COMM, SETT>(mg: &EventStream<<Mg<COMM, SETT> as Update>::Msg>, msg: String,
    default_answer: String) -> Option<String>
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
{
    let (blocking_input_dialog, rx) = BlockingInputDialog::new();
    let responder = Box::new(blocking_input_dialog);
    mg.emit(BlockingInput(responder, msg, default_answer));
    gtk::main();
    mg.emit(ResetInput);
    rx.try_recv().unwrap_or(None)
}

/// Ask a multiple-choice question to the user and block until the user provides it (or cancel).
pub fn blocking_question<COMM, SETT>(mg: &EventStream<<Mg<COMM, SETT> as Update>::Msg>, msg: String,
    choices: &[char]) -> Option<String>
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
{
    let (blocking_input_dialog, rx) = BlockingInputDialog::new();
    let responder = Box::new(blocking_input_dialog);
    mg.emit(BlockingQuestion(responder, msg, choices.to_vec()));
    gtk::main();
    mg.emit(ResetInput);
    rx.try_recv().unwrap_or(None)
}

/// Show a blocking yes/no question.
pub fn blocking_yes_no_question<COMM, SETT>(mg: &EventStream<<Mg<COMM, SETT> as Update>::Msg>, msg: String)
    -> bool
where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
{
    let (blocking_input_dialog, rx) = BlockingInputDialog::new();
    let responder = Box::new(blocking_input_dialog);
    mg.emit(BlockingYesNoQuestion(responder, msg));
    gtk::main();
    mg.emit(ResetInput);
    rx.try_recv() == Ok(Some("y".to_string()))
}

/// Ask a question to the user.
pub fn input<CALLBACK, COMM, SETT, WIDGET>(mg: &ContainerComponent<Mg<COMM, SETT>>, relm: &Relm<WIDGET>, msg: String,
    default_answer: String, callback: CALLBACK)
where CALLBACK: Fn(Option<String>) -> WIDGET::Msg + 'static,
      COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
      WIDGET: Widget + 'static,
{
    let responder = Box::new(InputDialog::new(relm, callback));
    mg.emit(Input(responder, msg, default_answer));
}

/// Ask a multiple-choice question to the user.
pub fn question<CALLBACK, COMM, SETT, WIDGET>(mg: &ContainerComponent<Mg<COMM, SETT>>, relm: &Relm<WIDGET>, msg: String,
    choices: &'static [char], callback: CALLBACK)
where CALLBACK: Fn(Option<String>) -> WIDGET::Msg + 'static,
      COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
      WIDGET: Widget + 'static,
{
    let responder = Box::new(InputDialog::new(relm, callback));
    mg.emit(Question(responder, msg, choices));
}

/// Show a yes/no question.
pub fn yes_no_question<CALLBACK, COMM, SETT, WIDGET>(mg: &ContainerComponent<Mg<COMM, SETT>>, relm: &Relm<WIDGET>,
    msg: String, callback: CALLBACK)
where CALLBACK: Fn(bool) -> WIDGET::Msg + 'static,
      COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
      SETT: Default + EnumMetaData + settings::Settings + SettingCompletion + 'static,
      WIDGET: Widget + 'static,
{
    let responder = Box::new(YesNoInputDialog::new(relm, callback));
    mg.emit(YesNoQuestion(responder, msg));
}
