use super::typography::{heading, text};
use crate::catalog::current_year_season;
use anilist_core::season::AnimeSeason;
use windows_reactor::*;

const EARLIEST_YEAR: u16 = 2015;
const SEASONS: [AnimeSeason; 4] = [
    AnimeSeason::Winter,
    AnimeSeason::Spring,
    AnimeSeason::Summer,
    AnimeSeason::Fall,
];

#[derive(Clone, PartialEq)]
pub struct Selection {
    pub year: u16,
    pub season: AnimeSeason,
    pub on_select: Callback<(u16, AnimeSeason)>,
}

struct SeasonPicker {
    year: u16,
    revision: u64,
    selection: Selection,
}
enum Message {
    Open,
    Year(u16),
    Select(AnimeSeason),
}

pub fn view(year: u16, season: AnimeSeason, on_select: Callback<(u16, AnimeSeason)>) -> View {
    View::component::<SeasonPicker>(Selection {
        year,
        season,
        on_select,
    })
}

impl Component for SeasonPicker {
    type Input = Selection;
    type Message = Message;

    fn create(input: &Selection, _: &ComponentContext<Self>) -> Self {
        Self {
            year: input.year,
            revision: 0,
            selection: input.clone(),
        }
    }

    fn input_changed(&mut self, input: &Selection, _: &ComponentContext<Self>) {
        self.selection = input.clone();
    }

    fn update(&mut self, message: Message, _: &ComponentContext<Self>) {
        match message {
            Message::Open => self.year = self.selection.year,
            Message::Year(year) => {
                self.year = year.clamp(EARLIEST_YEAR, current_year_season().0 + 1)
            }
            Message::Select(season) => {
                let _ = self.selection.on_select.call((self.year, season));
                // Retiring the flyout closes it even when the selected season did not change.
                self.revision += 1;
            }
        }
    }

    fn view(&self, input: &Selection, context: &mut ViewContext<Self>) -> View {
        let previous = self.year.saturating_sub(1);
        let next = self.year.saturating_add(1);
        let content = StackPanel::new().width(272.0).spacing(16.0).children((
            Grid::new()
                .columns([GridLength::Auto, GridLength::Star(1.0), GridLength::Auto])
                .children((
                    Button::new()
                        .style(ButtonStyle::Subtle)
                        .automation_name("前一年")
                        .is_enabled(self.year > EARLIEST_YEAR)
                        .on_click(context.callback(move |()| Message::Year(previous)))
                        .content(SymbolIcon::new().symbol(Symbol::Back)),
                    heading(self.year.to_string(), 20.0)
                        .grid_column(1)
                        .horizontal_alignment(HorizontalAlignment::Center),
                    Button::new()
                        .grid_column(2)
                        .style(ButtonStyle::Subtle)
                        .automation_name("後一年")
                        .is_enabled(self.year < current_year_season().0 + 1)
                        .on_click(context.callback(move |()| Message::Year(next)))
                        .content(SymbolIcon::new().symbol(Symbol::Forward)),
                )),
            Grid::new()
                .columns([GridLength::Star(1.0); 2])
                .rows([GridLength::Auto; 2])
                .row_spacing(8.0)
                .column_spacing(8.0)
                .keyed_children(SEASONS.into_iter().enumerate().map(|(index, season)| {
                    KeyedView::new(
                        index,
                        Button::new()
                            .grid_column((index % 2) as i32)
                            .grid_row((index / 2) as i32)
                            .horizontal_alignment(HorizontalAlignment::Stretch)
                            .style(if input.year == self.year && input.season == season {
                                ButtonStyle::Accent
                            } else {
                                ButtonStyle::Default
                            })
                            .automation_name(format!("選擇 {} {}", self.year, season))
                            .on_click(context.callback(move |()| Message::Select(season)))
                            .content(text(season.to_string())),
                    )
                })),
        ));
        View::keyed_fragment([KeyedView::new(
            self.revision,
            DropDownButton::new()
                .horizontal_alignment(HorizontalAlignment::Left)
                .automation_name("選擇年份與季度")
                .automation_id("SeasonSelector")
                .on_click(context.callback(|()| Message::Open))
                .content(heading(format!("{} {}", input.year, input.season), 28.0))
                .flyout_with(Flyout::rich(content).placement(FlyoutPlacement::Bottom)),
        )])
    }
}
