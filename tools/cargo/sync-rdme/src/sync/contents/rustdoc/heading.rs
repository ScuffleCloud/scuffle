use pulldown_cmark::{Event, Tag, TagEnd};

pub(super) fn convert<'a, 'b>(
    events: impl IntoIterator<Item = Event<'a>> + 'b,
) -> impl Iterator<Item = Event<'a>> + 'b {
    use pulldown_cmark::HeadingLevel::*;
    events.into_iter().map(|mut event| {
        match &mut event {
            Event::Start(Tag::Heading { level, .. }) | Event::End(TagEnd::Heading(level)) => {
                *level = match level {
                    H1 => H2,
                    H2 => H3,
                    H3 => H4,
                    H4 => H5,
                    H5 => H6,
                    H6 => H6,
                }
            }
            _ => {}
        }
        event
    })
}
