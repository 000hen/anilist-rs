use iced::{
    Element, Padding,
    widget::{Container, container},
};

pub fn badge<'a, Message>(component: impl Into<Element<'a, Message>>) -> Container<'a, Message>
where
    Message: Clone + 'a,
{
    container(component)
        .style(container::rounded_box)
        .padding(Padding::from([2, 4]))
}
