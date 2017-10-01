/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
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

use std::collections::HashMap;

use mg_settings::{
    self,
    Command,
    EnumFromStr,
    EnumMetaData,
    ParseResult,
    SettingCompletion,
    SpecialCommand,
};
use mg_settings::errors::Error;
use mg_settings::errors::ErrorType::{MissingArgument, NoCommand, Parse, UnknownCommand};
use mg_settings::Command::{App, Custom, Map, Set, Unmap};

use app::{
    Mg,
    Mode,
    COMPLETE_NEXT_COMMAND,
    COMPLETE_PREVIOUS_COMMAND,
    COPY,
    CUT,
    ENTRY_DELETE_NEXT_CHAR,
    ENTRY_DELETE_NEXT_WORD,
    ENTRY_DELETE_PREVIOUS_WORD,
    ENTRY_END,
    ENTRY_NEXT_CHAR,
    ENTRY_NEXT_WORD,
    ENTRY_PREVIOUS_CHAR,
    ENTRY_PREVIOUS_WORD,
    ENTRY_SMART_HOME,
    PASTE,
    PASTE_SELECTION,
};
use app::ActivationType::{self, Final};
use app::Msg::{
    self,
    CustomCommand,
    EnterNormalModeAndReset,
};
use app::status_bar::Msg::{
    Copy,
    Cut,
    DeleteNextChar,
    DeleteNextWord,
    DeletePreviousWord,
    End,
    NextChar,
    NextWord,
    Paste,
    PasteSelection,
    PreviousChar,
    PreviousWord,
    SmartHome,
};
use app::ShortcutCommand::{self, Complete, Incomplete};
use completion::completion_view::Msg::{SelectNext, SelectPrevious};

impl<COMM, SETT> Mg<COMM, SETT>
    where COMM: Clone + EnumFromStr + EnumMetaData + SpecialCommand + 'static,
          SETT: Default + EnumMetaData + mg_settings::settings::Settings + SettingCompletion + 'static,
{
    /// Convert an action String to a command String.
    pub fn action_to_command(&self, action: &str) -> ShortcutCommand {
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
            COMPLETE_NEXT_COMMAND => self.completion_view.emit(SelectNext),
            COMPLETE_PREVIOUS_COMMAND => self.completion_view.emit(SelectPrevious),
            COPY => self.status_bar.emit(Copy),
            CUT => self.status_bar.emit(Cut),
            ENTRY_DELETE_NEXT_CHAR => self.status_bar.emit(DeleteNextChar),
            ENTRY_DELETE_NEXT_WORD => self.status_bar.emit(DeleteNextWord),
            ENTRY_DELETE_PREVIOUS_WORD => self.status_bar.emit(DeletePreviousWord),
            ENTRY_END => self.status_bar.emit(End),
            ENTRY_NEXT_CHAR => self.status_bar.emit(NextChar),
            ENTRY_NEXT_WORD => self.status_bar.emit(NextWord),
            ENTRY_PREVIOUS_CHAR => self.status_bar.emit(PreviousChar),
            ENTRY_PREVIOUS_WORD => self.status_bar.emit(PreviousWord),
            ENTRY_SMART_HOME => self.status_bar.emit(SmartHome),
            PASTE => self.status_bar.emit(Paste),
            PASTE_SELECTION => self.status_bar.emit(PasteSelection),
            _ => unreachable!(),
        }
    }

    /// Call the callback with the command or show an error if the command cannot be parsed.
    fn call_command(&mut self, command: Command<COMM>) {
        match command {
            App(command) => self.app_command(&command),
            Custom(command) => self.model.relm.stream().emit(CustomCommand(command)),
            Map { action, keys, mode } => {
                let mode_mappings = self.model.mappings.entry(self.model.modes[mode.as_str()].name)
                    .or_insert_with(HashMap::new);
                mode_mappings.insert(keys, action);
            },
            Set(name, value) => {
                match SETT::to_variant(&name, value) {
                    Ok(setting) => self.set_setting(setting),
                    Err(error) => {
                        self.error(Error::Msg("Error setting value".to_string()));
                        error!("{}", error);
                    },
                }
                self.return_to_normal_mode();
            },
            Unmap { keys, mode } => {
                let mode_mappings = self.model.mappings.entry(self.model.modes[mode.as_str()].name)
                    .or_insert_with(HashMap::new);
                mode_mappings.remove(&keys);
            },
        }
    }

    /// Handle the command entry activate event.
    pub fn command_activate(&mut self, input: Option<String>) {
        let current_mode = self.model.current_mode.get();
        let message =
            if current_mode == Mode::Input || current_mode == Mode::BlockingInput {
                let mut should_reset = false;
                if let Some(callback) = self.model.input_callback.take() {
                    self.model.answer = input.clone();
                    callback(input, self.model.shortcut_pressed);
                    should_reset = true;
                }
                self.model.choices.clear();
                if should_reset {
                    Some(EnterNormalModeAndReset)
                }
                else {
                    None
                }
            }
            else {
                self.handle_command(input, true, None)
            };
        if let Some(message) = message {
            self.model.relm.stream().emit(message);
        }
    }

    /// Execute the commands and show the errors contained in the parse result.
    pub fn execute_commands(&mut self, mut parse_result: ParseResult<COMM>, activated: bool) {
        for command in parse_result.commands.drain(..) {
            self.call_command(command);
        }
        for error in parse_result.errors.drain(..) {
            self.show_parse_error(error);
        }
        if activated {
            self.return_to_normal_mode();
        }
    }

    /// Handle the command activate event.
    pub fn handle_command(&mut self, command: Option<String>, activated: bool, prefix: Option<u32>)
        -> Option<Msg<COMM, SETT>>
    {
        if let Some(command) = command {
            if self.is_normal_command() || !activated {
                let parse_result = self.model.settings_parser.parse_line(&command, prefix);
                self.execute_commands(parse_result, activated);
            }
            else {
                // If activated is true, it means the user pressed Enter to finish the special
                // command. If it was false, that means that the user activated a command via a
                // shortcut.
                return self.handle_special_command(Final, &command);
            }
        }
        None
    }

    /// Handle a special command activate or key press event.
    pub fn handle_special_command(&mut self, activation_type: ActivationType, command: &str) -> Option<Msg<COMM, SETT>> {
        if let Ok(special_command) = COMM::identifier_to_command(self.model.current_command_mode, command) {
            if activation_type == Final {
                self.return_to_normal_mode();
            }
            Some(CustomCommand(special_command))
        }
        else {
            None
        }
    }

    fn show_parse_error(&mut self, error: Error) {
        if let Error::Parse(ref parse_error) = error {
            let message =
                match parse_error.typ {
                    MissingArgument => "Argument required".to_string(),
                    NoCommand => return,
                    Parse => format!("Parse error: unexpected {}, expecting: {}", parse_error.unexpected,
                                     parse_error.expected),
                    UnknownCommand => format!("Not a command: {}", parse_error.unexpected),
                };
            self.error(Error::Msg(message));
        }

        error!("{}", error);
    }
}
