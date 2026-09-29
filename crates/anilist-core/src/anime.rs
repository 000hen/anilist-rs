use std::fmt::{self, Display};

use crate::time::AnimeTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimeSite {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimeStreaming {
    pub name: String,
    pub url: String,
    pub logo: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anime {
    pub id: String,

    pub name: String,
    pub description: String,
    pub on_air_time: Option<AnimeTime>,
    pub is_adult: bool,

    pub image: Option<String>,
    pub banner: Option<String>,

    pub cast: Vec<String>,
    pub genres: Vec<String>,

    pub streaming: Vec<AnimeStreaming>,
    pub site: Vec<AnimeSite>,
}

impl Display for Anime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "+-- Anime: {}", self.name)?;
        writeln!(f, "|   ID: {}", self.id)?;
        writeln!(f, "|   Adult: {}", if self.is_adult { "yes" } else { "no" })?;

        match &self.on_air_time {
            Some(time) => match time.minute {
                Some(minute) => writeln!(
                    f,
                    "|   Airing: {} {:02}:{:02} ({})",
                    time.week,
                    minute.get() / 60,
                    minute.get() % 60,
                    time.zone
                )?,
                None => writeln!(f, "|   Airing: {} time TBD ({})", time.week, time.zone)?,
            },
            None => writeln!(f, "|   Airing: not scheduled")?,
        }

        writeln!(f, "|-- Description")?;
        if self.description.is_empty() {
            writeln!(f, "|   (none)")?;
        } else {
            for line in self.description.lines() {
                writeln!(f, "|   {line}")?;
            }
        }

        writeln!(f, "|-- Artwork")?;
        writeln!(
            f,
            "|   Image: {}",
            self.image.as_deref().unwrap_or("(none)")
        )?;
        writeln!(
            f,
            "|   Banner: {}",
            self.banner.as_deref().unwrap_or("(none)")
        )?;

        write_string_list(f, "Cast", &self.cast)?;
        write_string_list(f, "Genres", &self.genres)?;

        writeln!(f, "|-- Streaming ({})", self.streaming.len())?;
        if self.streaming.is_empty() {
            writeln!(f, "|   (none)")?;
        } else {
            for stream in &self.streaming {
                writeln!(f, "|   - {}: {}", stream.name, stream.url)?;
                writeln!(f, "|     Logo: {}", stream.logo)?;
            }
        }

        writeln!(f, "`-- Sites ({})", self.site.len())?;
        if self.site.is_empty() {
            writeln!(f, "    (none)")?;
        } else {
            for site in &self.site {
                writeln!(f, "    - {}: {}", site.title, site.url)?;
            }
        }

        Ok(())
    }
}

fn write_string_list(f: &mut fmt::Formatter<'_>, title: &str, values: &[String]) -> fmt::Result {
    writeln!(f, "|-- {title} ({})", values.len())?;
    if values.is_empty() {
        writeln!(f, "|   (none)")?;
    } else {
        for value in values {
            writeln!(f, "|   - {value}")?;
        }
    }
    Ok(())
}
