use anilist_core::{
    ScheduleDay, anime::Anime, get_week_order, season::AnimeSeason, time::AnimeTime,
};
use chrono::{Datelike, Local, Weekday};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CatalogSummary {
    pub total: usize,
    pub today: usize,
    pub streamable: usize,
}

#[derive(Debug, Clone)]
pub struct DaySection {
    pub day: ScheduleDay,
    pub animes: Vec<Anime>,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub sections: Vec<DaySection>,
    pub summary: CatalogSummary,
}

impl Catalog {
    pub fn new(animes: Vec<Anime>, today: Weekday) -> Self {
        let summary = CatalogSummary {
            total: animes.len(),
            today: animes
                .iter()
                .filter(|anime| {
                    anime
                        .on_air_time
                        .as_ref()
                        .is_some_and(|time| time.week == today)
                })
                .count(),
            streamable: animes
                .iter()
                .filter(|anime| !anime.streaming.is_empty())
                .count(),
        };

        let mut sections: Vec<_> = get_week_order(today)
            .into_iter()
            .map(|day| DaySection {
                day,
                animes: Vec::new(),
            })
            .collect();

        for anime in animes {
            let day = anime
                .on_air_time
                .as_ref()
                .map(|time| ScheduleDay::Weekday(time.week))
                .unwrap_or(ScheduleDay::Unknown);
            if let Some(section) = sections.iter_mut().find(|section| section.day == day) {
                section.animes.push(anime);
            }
        }

        for section in &mut sections {
            section.animes.sort_by(|a, b| {
                match (&a.on_air_time, &b.on_air_time) {
                    (Some(a), Some(b)) => AnimeTime::airing_cmp(a, b),
                    _ => std::cmp::Ordering::Equal,
                }
                .then_with(|| a.name.cmp(&b.name))
            });
        }
        sections.retain(|section| !section.animes.is_empty());

        Self { sections, summary }
    }

    pub fn search(&self, query: &str) -> Vec<&Anime> {
        let query = query.trim().to_lowercase();
        self.sections
            .iter()
            .flat_map(|section| &section.animes)
            .filter(|anime| matches_query(anime, &query))
            .collect()
    }
}

pub fn current_year_season() -> (u16, AnimeSeason) {
    let now = Local::now();
    let year = u16::try_from(now.year()).expect("current year must fit in u16");
    let season = AnimeSeason::try_from(now.month() as u8).expect("month should map to a season");
    (year, season)
}

pub fn schedule_text(anime: &Anime) -> String {
    match &anime.on_air_time {
        Some(time) => match time.minute {
            Some(minute) => format!("{} {minute}", time.week),
            None => format!("{} 時間未定", time.week),
        },
        None => "時間未定".to_owned(),
    }
}

fn matches_query(anime: &Anime, query: &str) -> bool {
    query.is_empty()
        || anime.name.to_lowercase().contains(query)
        || anime.description.to_lowercase().contains(query)
        || anime
            .genres
            .iter()
            .any(|genre| genre.to_lowercase().contains(query))
        || anime
            .cast
            .iter()
            .any(|member| member.to_lowercase().contains(query))
}

#[cfg(test)]
mod tests {
    use anilist_core::{anime::AnimeStreaming, minute::Minute};

    use super::*;

    fn anime(name: &str, week: Option<Weekday>, minute: Option<u16>) -> Anime {
        Anime {
            id: name.to_owned(),
            name: name.to_owned(),
            description: String::new(),
            on_air_time: week.map(|week| AnimeTime {
                week,
                minute: minute.and_then(Minute::new),
                zone: "Asia/Taipei".to_owned(),
            }),
            is_adult: false,
            image: None,
            banner: None,
            cast: Vec::new(),
            genres: Vec::new(),
            streaming: Vec::new(),
            site: Vec::new(),
        }
    }

    #[test]
    fn groups_from_today_and_places_unknown_last() {
        let catalog = Catalog::new(
            vec![
                anime("unknown", None, None),
                anime("tomorrow", Some(Weekday::Sun), Some(60)),
                anime("late", Some(Weekday::Sat), Some(120)),
                anime("early", Some(Weekday::Sat), Some(60)),
                anime("tbd", Some(Weekday::Sat), None),
            ],
            Weekday::Sat,
        );

        assert_eq!(catalog.sections.len(), 3);
        assert_eq!(catalog.sections[0].day, ScheduleDay::Weekday(Weekday::Sat));
        assert_eq!(
            catalog.sections[0]
                .animes
                .iter()
                .map(|anime| anime.name.as_str())
                .collect::<Vec<_>>(),
            ["early", "late", "tbd"]
        );
        assert_eq!(catalog.sections[1].day, ScheduleDay::Weekday(Weekday::Sun));
        assert_eq!(catalog.sections[2].day, ScheduleDay::Unknown);
    }

    #[test]
    fn summary_counts_today_and_streaming() {
        let mut streamable = anime("streamable", Some(Weekday::Tue), None);
        streamable.streaming.push(AnimeStreaming {
            name: "Service".to_owned(),
            url: "https://example.com".to_owned(),
            logo: String::new(),
        });
        let catalog = Catalog::new(vec![streamable, anime("unknown", None, None)], Weekday::Tue);
        assert_eq!(
            catalog.summary,
            CatalogSummary {
                total: 2,
                today: 1,
                streamable: 1,
            }
        );
    }

    #[test]
    fn search_matches_name_description_genre_and_cast() {
        let mut item = anime("Example Show", Some(Weekday::Mon), Some(90));
        item.description = "An ocean story".to_owned();
        item.genres.push("Adventure".to_owned());
        item.cast.push("Alice".to_owned());
        let catalog = Catalog::new(vec![item], Weekday::Mon);
        for query in ["show", "OCEAN", "adventure", "alice", "  ExAmPlE  ", ""] {
            assert_eq!(catalog.search(query).len(), 1);
        }
        assert!(catalog.search("missing").is_empty());
    }

    #[test]
    fn schedule_formats_known_and_unknown_times() {
        assert_eq!(
            schedule_text(&anime("known", Some(Weekday::Mon), Some(90))),
            "Mon 01:30"
        );
        assert_eq!(
            schedule_text(&anime("unknown time", Some(Weekday::Mon), None)),
            "Mon 時間未定"
        );
        assert_eq!(schedule_text(&anime("unknown day", None, None)), "時間未定");
    }
}
