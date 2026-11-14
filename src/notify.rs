//! Desktop notifications over D-Bus (mako, dunst, GNOME and friends), with
//! "Done" and "In 1 hour" buttons where the notification server offers them.

use std::collections::HashMap;

use iced::futures::{SinkExt, Stream, StreamExt};
use zbus::zvariant::Value;

use crate::i18n::t;
use crate::model::Id;

const BUS: &str = "org.freedesktop.Notifications";
const PATH: &str = "/org/freedesktop/Notifications";

pub const DONE: &str = "done";
pub const LATER: &str = "later";

/// One notification to show. `task` adds the action buttons.
pub struct Note {
    pub body: String,
    pub task: Option<Id>,
}

/// Shows each note and returns the server's id for those tied to a to-do,
/// so a pressed button can be traced back to it.
pub fn show(notes: Vec<Note>) -> Vec<(u32, Id)> {
    let Ok(connection) = zbus::blocking::Connection::session() else { return Vec::new() };
    let mut sent = Vec::new();
    for note in notes {
        let hints: HashMap<&str, Value<'_>> = HashMap::new();
        let actions: Vec<&str> = if note.task.is_some() { vec![DONE, t("Done"), LATER, t("In 1 hour")] } else { Vec::new() };
        let reply = connection.call_method(
            Some(BUS),
            PATH,
            Some(BUS),
            "Notify",
            &("gotodo", 0u32, "gotodo", t("Reminder"), note.body.as_str(), actions, hints, -1i32),
        );
        if let (Ok(reply), Some(task)) = (reply, note.task) {
            if let Ok(id) = reply.body().deserialize::<u32>() {
                sent.push((id, task));
            }
        }
    }
    sent
}

/// Every pressed notification button: the notification id and the action key.
pub fn actions() -> impl Stream<Item = (u32, String)> {
    iced::stream::channel(8, async |mut output: iced::futures::channel::mpsc::Sender<(u32, String)>| {
        let Ok(connection) = zbus::Connection::session().await else { return };
        let Ok(rule) = zbus::MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .interface(BUS)
            .and_then(|rule| rule.member("ActionInvoked"))
            .map(|rule| rule.build())
        else {
            return;
        };
        let Ok(mut stream) = zbus::MessageStream::for_match_rule(rule, &connection, None).await else { return };
        while let Some(Ok(message)) = stream.next().await {
            if let Ok((id, key)) = message.body().deserialize::<(u32, String)>() {
                let _ = output.send((id, key)).await;
            }
        }
    })
}
