use super::shell;
use crate::{
    app::{Application, Message},
    pages::schedule,
};
use windows_reactor::*;

pub fn view(app: &Application, context: &mut ViewContext<Application>) -> View {
    context.window_title("Anilist · 動畫時間表");
    context.window_visuals(shell::visuals(1200.0, 860.0));
    context.on_window_size(context.callback(|size: WindowSize| Message::Resized(size.width)));
    shell::frame_with_title_bar(
        TitleBar::new()
            .title("Anilist")
            .min_height(48.0)
            .preferred_height(WindowTitleBarHeight::Tall)
            .slots([SlotView::new(
                TitleBarSlot::Content,
                crate::components::title_search::view(
                    &app.source_query,
                    app.width,
                    context.callback(Message::SourceQuery),
                    context.callback(|()| Message::SearchSource),
                ),
            )]),
        schedule::view(
            schedule::Page {
                year: app.year,
                season: app.season,
                catalog: &app.catalog,
                query: &app.query,
                search_term: app.search_term.as_deref(),
                day: app.day,
                loading: app.loading,
                error: app.error.as_deref(),
                width: app.width,
            },
            context.callback(Message::Page),
        ),
    )
}
