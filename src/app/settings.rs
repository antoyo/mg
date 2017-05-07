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

use std::collections::HashMap;

use mg_settings::{EnumMetaData, MetaData, SettingCompletion, Value};
use mg_settings::error::SettingError;
use mg_settings::settings;

#[doc(hidden)]
#[derive(Default)]
pub struct NoSettings;

#[doc(hidden)]
#[derive(Clone)]
pub enum NoSettingsVariant { }

impl settings::Settings for NoSettings {
    type Variant = NoSettingsVariant;

    fn to_variant(name: &str, _value: Value) -> Result<Self::Variant, SettingError> {
        Err(SettingError::UnknownSetting(name.to_string()))
    }

    fn set_value(&mut self, _value: Self::Variant) {
    }
}

impl EnumMetaData for NoSettings {
    fn get_metadata() -> HashMap<String, MetaData> {
        HashMap::new()
    }
}

impl SettingCompletion for NoSettings {
    fn get_value_completions() -> HashMap<String, Vec<String>> {
        HashMap::new()
    }
}
