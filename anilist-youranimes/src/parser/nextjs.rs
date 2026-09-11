use scraper::{Html, Selector};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::errors::YourAnimesError;

const NEXTJS_SCRIPT_HANDLER: &str = "self.__next_f.push(";
const NEXTJS_SCRIPT_SUFFIX: &str = ")";

#[derive(Debug, Clone, Copy)]
pub enum JsonPath<'a> {
    Index(usize),
    Key(&'a str),
}

pub struct NextJsExtractor;

impl NextJsExtractor {
    pub fn extract<T>(
        body: &Html,
        matches: &str,
        path: &[JsonPath<'_>],
    ) -> Result<T, YourAnimesError>
    where
        T: DeserializeOwned,
    {
        let frame = Self::find_frame(body, matches)?;

        let payload =
            frame
                .get(1)
                .and_then(Value::as_str)
                .ok_or(YourAnimesError::InvalidResponse {
                    context: "Next.js frame has no string payload",
                })?;

        let (_, payload) = payload
            .split_once(':')
            .ok_or(YourAnimesError::InvalidResponse {
                context: "Next.js frame payload has no separator",
            })?;

        let embedded = serde_json::Deserializer::from_str(payload)
            .into_iter::<Value>()
            .next()
            .ok_or(YourAnimesError::InvalidResponse {
                context: "Next.js frame contains no embedded JSON",
            })?
            .map_err(|source| YourAnimesError::Json {
                context: "Next.js embedded JSON",
                source,
            })?;

        let target = Self::get_path(&embedded, path).ok_or(YourAnimesError::InvalidResponse {
            context: "Next.js target data was not found",
        })?;

        serde_json::from_value(target.clone()).map_err(|source| YourAnimesError::Json {
            context: "Next.js target data",
            source,
        })
    }

    fn find_frame(body: &Html, matches: &str) -> Result<Value, YourAnimesError> {
        let selector =
            Selector::parse("script").expect("static script selector should always be valid");

        for element in body.select(&selector) {
            let script = element.inner_html();

            let Some(content) = script
                .strip_prefix(NEXTJS_SCRIPT_HANDLER)
                .and_then(|content| content.strip_suffix(NEXTJS_SCRIPT_SUFFIX))
            else {
                continue;
            };

            if !content.contains(matches) {
                continue;
            }

            return serde_json::from_str(content).map_err(|source| YourAnimesError::Json {
                context: "Next.js data frame",
                source,
            });
        }

        Err(YourAnimesError::InvalidResponse {
            context: "matching Next.js data frame was not found",
        })
    }

    fn get_path<'a>(mut value: &'a Value, path: &[JsonPath<'_>]) -> Option<&'a Value> {
        for segment in path {
            value = match segment {
                JsonPath::Index(index) => value.get(*index)?,
                JsonPath::Key(key) => value.get(*key)?,
            };
        }

        Some(value)
    }
}
