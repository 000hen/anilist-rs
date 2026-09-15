use anilist_core::{
    ScheduleDay as CoreScheduleDay,
    anime::{Anime as CoreAnime, AnimeSite as CoreAnimeSite, AnimeStreaming as CoreAnimeStreaming},
    minute::Minute as CoreMinute,
    season::AnimeSeason as CoreAnimeSeason,
    time::AnimeTime as CoreAnimeTime,
};
use chrono::Weekday as CoreWeekday;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl From<CoreWeekday> for Weekday {
    fn from(value: CoreWeekday) -> Self {
        match value {
            CoreWeekday::Mon => Self::Monday,
            CoreWeekday::Tue => Self::Tuesday,
            CoreWeekday::Wed => Self::Wednesday,
            CoreWeekday::Thu => Self::Thursday,
            CoreWeekday::Fri => Self::Friday,
            CoreWeekday::Sat => Self::Saturday,
            CoreWeekday::Sun => Self::Sunday,
        }
    }
}

impl From<Weekday> for CoreWeekday {
    fn from(value: Weekday) -> Self {
        match value {
            Weekday::Monday => Self::Mon,
            Weekday::Tuesday => Self::Tue,
            Weekday::Wednesday => Self::Wed,
            Weekday::Thursday => Self::Thu,
            Weekday::Friday => Self::Fri,
            Weekday::Saturday => Self::Sat,
            Weekday::Sunday => Self::Sun,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct Minute {
    pub value: u16,
}

impl From<CoreMinute> for Minute {
    fn from(value: CoreMinute) -> Self {
        Self { value: value.get() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum AnimeSeason {
    Winter,
    Spring,
    Summer,
    Fall,
}

impl From<CoreAnimeSeason> for AnimeSeason {
    fn from(value: CoreAnimeSeason) -> Self {
        match value {
            CoreAnimeSeason::Winter => Self::Winter,
            CoreAnimeSeason::Spring => Self::Spring,
            CoreAnimeSeason::Summer => Self::Summer,
            CoreAnimeSeason::Fall => Self::Fall,
        }
    }
}

impl From<AnimeSeason> for CoreAnimeSeason {
    fn from(value: AnimeSeason) -> Self {
        match value {
            AnimeSeason::Winter => Self::Winter,
            AnimeSeason::Spring => Self::Spring,
            AnimeSeason::Summer => Self::Summer,
            AnimeSeason::Fall => Self::Fall,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum ScheduleDay {
    Weekday { day: Weekday },
    Unknown,
}

impl From<CoreScheduleDay> for ScheduleDay {
    fn from(value: CoreScheduleDay) -> Self {
        match value {
            CoreScheduleDay::Weekday(day) => Self::Weekday { day: day.into() },
            CoreScheduleDay::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct AnimeTime {
    pub week: Weekday,
    pub minute: Option<Minute>,
    pub zone: String,
}

impl From<CoreAnimeTime> for AnimeTime {
    fn from(value: CoreAnimeTime) -> Self {
        Self {
            week: value.week.into(),
            minute: value.minute.map(Into::into),
            zone: value.zone,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct AnimeSite {
    pub title: String,
    pub url: String,
}

impl From<CoreAnimeSite> for AnimeSite {
    fn from(value: CoreAnimeSite) -> Self {
        Self {
            title: value.title,
            url: value.url,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct AnimeStreaming {
    pub name: String,
    pub url: String,
    pub logo: String,
}

impl From<CoreAnimeStreaming> for AnimeStreaming {
    fn from(value: CoreAnimeStreaming) -> Self {
        Self {
            name: value.name,
            url: value.url,
            logo: value.logo,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
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
    pub sites: Vec<AnimeSite>,
}

impl From<CoreAnime> for Anime {
    fn from(value: CoreAnime) -> Self {
        Self {
            id: value.id,

            name: value.name,
            description: value.description,
            on_air_time: value.on_air_time.map(Into::into),
            is_adult: value.is_adult,

            image: value.image,
            banner: value.banner,

            cast: value.cast,
            genres: value.genres,

            streaming: value.streaming.into_iter().map(Into::into).collect(),

            sites: value.site.into_iter().map(Into::into).collect(),
        }
    }
}
