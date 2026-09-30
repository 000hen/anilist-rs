use super::shell;
use crate::{
    app::{Application, MainSurface, Message},
    pages::{schedule, search},
};
use windows_reactor::*;

pub fn view(app: &Application, context: &mut ViewContext<Application>) -> View {
    context.window_title("Anilist · 動畫時間表");
    context.window_visuals(shell::visuals(1200.0, 860.0));
    context.on_window_size(context.callback(|size: WindowSize| Message::Resized(size.width)));
    let content = match app.surface {
        MainSurface::Schedule => schedule::view(
            schedule::Page {
                year: app.year,
                season: app.season,
                catalog: &app.catalog,
                loading: app.loading,
                error: app.error.as_deref(),
                width: app.width,
            },
            context.callback(Message::Page),
        ),
        MainSurface::Search => search::view(
            search::Page {
                query: &app.source_query,
                term: app.search_term.as_deref(),
                results: &app.search_results,
                loading: app.search_loading,
                error: app.search_error.as_deref(),
                width: app.width,
            },
            context.callback(Message::SourceQuery),
            context.callback(|()| Message::SearchSource),
            context.callback(|()| Message::Back),
            context.callback(|anime| Message::Page(schedule::Event::Open(anime))),
        ),
    };
    shell::frame_with_title_bar(
        TitleBar::new()
            .title("Anilist")
            .preferred_height(WindowTitleBarHeight::Standard)
            .into(),
        content,
    )
}
