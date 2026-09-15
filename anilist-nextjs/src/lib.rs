//! Extract Next.js hydration data without executing JavaScript.
//!
//! Inspired by <https://github.com/novitae/njsparser>. Flight transport chunks
//! are joined before decoding records, including byte-counted UTF-8 text.
//! React references (such as `$L1`) remain strings; this is a data parser,
//! not a React runtime. Binary typed-array records are not supported.

mod error;
mod flight;

pub use error::ParseError;
pub use flight::FlightRecord;

use html5tokenizer::{NaiveParser, Token};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Hydration data from the Pages Router and/or App Router.
#[derive(Debug, Clone)]
pub struct NextJsData {
    pub next_data: Option<Value>,
    /// Records in wire order, preserving repeated and absent IDs.
    pub flight: Vec<FlightRecord>,
}

impl NextJsData {
    pub fn parse(html: &str) -> Result<Self, ParseError> {
        let mut next_data = None;
        let mut bytes = Vec::new();
        let mut has_flight = false;
        // Reading a &str is infallible. The tokenizer handles script-data and
        // raw-text states without constructing a DOM or decoding script text.
        let mut tokens = NaiveParser::new(html).flatten();
        while let Some(token) = tokens.next() {
            let Token::StartTag(tag) = token else {
                continue;
            };
            if tag.name != "script" {
                continue;
            }
            let mut script = String::new();
            for token in tokens.by_ref() {
                match token {
                    Token::Char(c) => script.push(c),
                    Token::EndTag(tag) if tag.name == "script" => break,
                    Token::EndOfFile => break,
                    _ => {}
                }
            }
            if tag.attributes.get("id") == Some("__NEXT_DATA__") {
                next_data = Some(error::json(&script, "Next.js __NEXT_DATA__")?);
            } else if tag.attributes.get("src").is_none() {
                has_flight |= flight::read_script(&script, &mut bytes)?;
            }
        }
        if next_data.is_none() && !has_flight {
            return Err(ParseError::MissingData);
        }
        Ok(Self {
            next_data,
            flight: flight::parse_records(&bytes)?,
        })
    }

    /// Find the first model record with this RFC 6901 JSON pointer, then try
    /// `__NEXT_DATA__`. Tagged metadata records are excluded from model lookup.
    /// Pointers are relative to each record's root; references are not followed.
    pub fn find(&self, pointer: &str) -> Option<&Value> {
        self.flight
            .iter()
            .filter(|record| record.tag.is_none())
            .find_map(|record| record.value.pointer(pointer))
            .or_else(|| self.next_data.as_ref()?.pointer(pointer))
    }

    pub fn deserialize<T: DeserializeOwned>(&self, pointer: &str) -> Result<T, ParseError> {
        let value = self.find(pointer).ok_or_else(|| ParseError::MissingPath {
            pointer: pointer.to_owned(),
        })?;
        serde_json::from_value(value.clone()).map_err(|source| ParseError::Json {
            context: "Next.js target data",
            source,
        })
    }
}
