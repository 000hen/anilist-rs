use windows_reactor::*;

pub fn view(
    query: &str,
    width: f64,
    on_change: Callback<String>,
    on_submit: Callback<String>,
) -> View {
    AutoSuggestBox::new()
        .text(query)
        .placeholder_text("搜尋動畫，按 Enter 搜尋")
        .width((width - 280.0).clamp(180.0, 480.0))
        .vertical_alignment(VerticalAlignment::Center)
        .automation_name("搜尋動畫")
        .on_text_changed(on_change)
        .on_query_submitted(on_submit)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    use windows_reactor::test::{
        Command, EventId, EventPayload, PropertyId, PropertyValue, Pump, QueuedEvent,
        RecordingRuntime,
    };

    #[test]
    fn native_submission_uses_query_payload_without_a_search_button() {
        let submitted = Rc::new(RefCell::new(Vec::new()));
        let result = submitted.clone();
        let mut pump = Pump::new(RecordingRuntime::default());
        pump.mount_view(view(
            "previous text",
            1200.0,
            Callback::new(|_| {}),
            Callback::new(move |query| result.borrow_mut().push(query)),
        ))
        .unwrap();
        let node = pump
            .runtime()
            .commands()
            .iter()
            .flatten()
            .find_map(|command| match command {
                Command::SetProperty {
                    node,
                    property: PropertyId::AutoSuggestBoxText,
                    ..
                } => Some(*node),
                _ => None,
            })
            .unwrap();
        let event = EventId::AutoSuggestBoxQuerySubmitted;
        let revision = pump.event_revision(node, event).unwrap();
        pump.queue_event(QueuedEvent::new(
            node,
            event,
            revision,
            EventPayload::Str("submitted text".into()),
        ));
        pump.dispatch_events().unwrap();
        assert_eq!(*submitted.borrow(), vec!["submitted text"]);
        assert!(!pump.runtime().commands().iter().flatten().any(|command| matches!(command,
            Command::SetProperty { property: PropertyId::AutomationName, value: PropertyValue::Str(value), .. } if value == "SubmitSourceSearch")));
        pump.update_view(View::empty()).unwrap();
        // Retired controls must not deliver an already queued native event.
        pump.queue_event(QueuedEvent::new(
            node,
            event,
            revision,
            EventPayload::Str("stale".into()),
        ));
        pump.dispatch_events().unwrap();
        assert_eq!(submitted.borrow().len(), 1);
    }
}
