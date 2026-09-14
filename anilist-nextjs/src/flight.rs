use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;

use crate::{ParseError, error};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FlightRecord {
    /// Hexadecimal wire ID converted to an integer; hints may have no ID.
    pub id: Option<u64>,
    /// `None` for a model, otherwise the wire tag (e.g. I, HL, D or T).
    pub tag: Option<String>,
    /// JSON payload, or a JSON string for a length-prefixed text record.
    pub value: Value,
}

fn transport(reason: &'static str) -> ParseError {
    ParseError::Transport { reason }
}

// Match only Next.js statements at the start of the remaining script. Never
// search arbitrary JavaScript strings for a push marker or evaluate page code.
fn push_argument(script: &str) -> Option<&str> {
    let mut rest = script;
    let tokens: &[&str] = if rest.starts_with('(') {
        &[
            "(", "self", ".", "__next_f", "=", "self", ".", "__next_f", "||", "[", "]", ")", ".",
            "push", "(",
        ]
    } else {
        &["self", ".", "__next_f", ".", "push", "("]
    };
    for token in tokens {
        rest = rest.trim_start().strip_prefix(token)?;
    }
    Some(rest.trim_start())
}

pub(crate) fn read_script(script: &str, bytes: &mut Vec<u8>) -> Result<bool, ParseError> {
    let mut rest = script.trim_start();
    let mut found = false;
    while let Some(argument) = push_argument(rest) {
        found = true;
        let mut stream = serde_json::Deserializer::from_str(argument).into_iter::<Value>();
        let value = stream
            .next()
            .ok_or_else(|| transport("missing push argument"))?
            .map_err(|source| ParseError::Json {
                context: "Next.js Flight segment",
                source,
            })?;
        rest = argument[stream.byte_offset()..]
            .trim_start()
            .strip_prefix(')')
            .ok_or_else(|| transport("missing closing push parenthesis"))?;
        let segment = value
            .as_array()
            .ok_or_else(|| transport("segment must be an array"))?;
        match segment.first().and_then(Value::as_u64) {
            Some(0) if segment.len() == 1 => bytes.clear(),
            Some(1 | 3) if segment.len() == 2 => {
                let payload = segment[1]
                    .as_str()
                    .ok_or_else(|| transport("payload must be a string"))?;
                if segment[0] == 1 {
                    bytes.extend_from_slice(payload.as_bytes());
                } else {
                    bytes.extend(
                        STANDARD
                            .decode(payload)
                            .map_err(|_| transport("invalid base64 payload"))?,
                    );
                }
            }
            Some(2) if segment.len() == 2 => {} // Form state is not Flight data.
            _ => return Err(transport("unknown segment type or invalid segment shape")),
        }
        rest = rest.trim_start();
        if let Some(next) = rest.strip_prefix(';') {
            rest = next.trim_start();
        }
    }
    Ok(found)
}

pub(crate) fn parse_records(bytes: &[u8]) -> Result<Vec<FlightRecord>, ParseError> {
    let mut result = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        if bytes[pos] == b'\n' || bytes[pos] == b'\r' {
            pos += 1;
            continue;
        }
        let offset = pos;
        let invalid = |reason| ParseError::Record { offset, reason };
        let colon = bytes[pos..]
            .iter()
            .position(|&b| b == b':')
            .ok_or_else(|| invalid("missing record separator"))?
            + pos;
        let id = if colon == pos {
            None
        } else {
            Some(hex(&bytes[pos..colon]).ok_or_else(|| invalid("invalid hexadecimal ID"))?)
        };
        pos = colon + 1;
        let first = *bytes.get(pos).ok_or_else(|| invalid("missing payload"))?;
        // Text lengths may start with A-F, so consume T separately from tags.
        if first == b'T' {
            pos += 1;
            let comma = bytes[pos..]
                .iter()
                .position(|&b| b == b',')
                .ok_or_else(|| invalid("missing text length separator"))?
                + pos;
            let length = hex(&bytes[pos..comma])
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| invalid("invalid text length"))?;
            pos = comma + 1;
            let end = pos
                .checked_add(length)
                .ok_or_else(|| invalid("text length overflow"))?;
            let text = bytes
                .get(pos..end)
                .and_then(|s| std::str::from_utf8(s).ok())
                .ok_or_else(|| invalid("truncated text or invalid UTF-8"))?;
            result.push(FlightRecord {
                id,
                tag: Some("T".into()),
                value: Value::String(text.into()),
            });
            pos = end;
            continue;
        }
        if b"AOoUSsLlGgMV".contains(&first) {
            return Err(invalid("binary typed-array records are unsupported"));
        }
        let tag_start = pos;
        if first.is_ascii_uppercase() || matches!(first, b'r' | b'x') {
            pos += 1;
            if first == b'H' && bytes.get(pos).is_some_and(u8::is_ascii_alphabetic) {
                pos += 1;
            }
        }
        let tag =
            (pos > tag_start).then(|| String::from_utf8_lossy(&bytes[tag_start..pos]).into_owned());
        let end = bytes[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .map_or(bytes.len(), |n| pos + n);
        let payload = std::str::from_utf8(&bytes[pos..end])
            .map_err(|_| invalid("invalid UTF-8"))?
            .trim_end_matches('\r');
        let value =
            if payload.is_empty() && matches!(tag.as_deref(), Some("R" | "r" | "X" | "x" | "C")) {
                Value::Null
            } else {
                error::json(payload, "Next.js Flight record JSON")?
            };
        result.push(FlightRecord { id, tag, value });
        pos = end;
    }
    Ok(result)
}

fn hex(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    u64::from_str_radix(std::str::from_utf8(bytes).ok()?, 16).ok()
}
