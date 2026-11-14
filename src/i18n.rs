//! UI language: English source strings with a German table, chosen from the locale.

use std::sync::OnceLock;

static GERMAN: OnceLock<bool> = OnceLock::new();

fn is_german() -> bool {
    *GERMAN.get_or_init(|| {
        ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .find_map(|name| std::env::var(name).ok().filter(|v| !v.is_empty()))
            .is_some_and(|value| value.starts_with("de"))
    })
}

pub fn t(english: &'static str) -> &'static str {
    if !is_german() {
        return english;
    }
    TABLE.iter().find(|(en, _)| *en == english).map_or(english, |(_, de)| de)
}

/// Key names as printed on a German keyboard: Strg, Entf.
pub fn keys(text: &str) -> String {
    if !is_german() {
        return text.to_owned();
    }
    text.replace("Ctrl", "Strg").replace("Del", "Entf")
}

const WEEKDAYS: [(&str, &str); 7] = [
    ("Monday", "Montag"),
    ("Tuesday", "Dienstag"),
    ("Wednesday", "Mittwoch"),
    ("Thursday", "Donnerstag"),
    ("Friday", "Freitag"),
    ("Saturday", "Samstag"),
    ("Sunday", "Sonntag"),
];

const MONTHS: [(&str, &str); 12] = [
    ("January", "Januar"),
    ("February", "Februar"),
    ("March", "März"),
    ("April", "April"),
    ("May", "Mai"),
    ("June", "Juni"),
    ("July", "Juli"),
    ("August", "August"),
    ("September", "September"),
    ("October", "Oktober"),
    ("November", "November"),
    ("December", "Dezember"),
];

pub fn weekday_name(index: usize) -> &'static str {
    let (en, de) = WEEKDAYS[index % 7];
    if is_german() { de } else { en }
}

pub fn month_name(month: u32) -> &'static str {
    let (en, de) = MONTHS[(month.clamp(1, 12) - 1) as usize];
    if is_german() { de } else { en }
}

const TABLE: &[(&str, &str)] = &[
    ("Inbox", "Eingang"),
    ("Today", "Heute"),
    ("Tomorrow", "Morgen"),
    ("Upcoming", "Geplant"),
    ("Anytime", "Jederzeit"),
    ("Someday", "Irgendwann"),
    ("Logbook", "Logbuch"),
    ("Trash", "Papierkorb"),
    ("Deadlines", "Fristen"),
    ("Repeating", "Wiederkehrend"),
    ("All Projects", "Alle Projekte"),
    ("This Evening", "Heute Abend"),
    ("New To-Do", "Neues To-Do"),
    ("New Project", "Neues Projekt"),
    ("New Area", "Neuer Bereich"),
    ("New Heading", "Neue Überschrift"),
    ("New List", "Neue Liste"),
    ("Project name", "Projektname"),
    ("Area name", "Name des Bereichs"),
    ("Heading name", "Überschrift"),
    ("New name", "Neuer Name"),
    ("Notes", "Notizen"),
    ("Checklist", "Checkliste"),
    ("Add item", "Eintrag hinzufügen"),
    ("Tags", "Tags"),
    ("When", "Wann"),
    ("Deadline", "Frist"),
    ("Move", "Verschieben"),
    ("Repeat", "Wiederholen"),
    ("Search", "Suchen"),
    ("Settings", "Einstellungen"),
    ("Lock", "Sperren"),
    ("Shortcuts", "Tastenkürzel"),
    ("Commands", "Befehle"),
    ("Actions", "Aktionen"),
    ("Complete", "Erledigen"),
    ("Cancel to-do", "Abbrechen"),
    ("Duplicate", "Duplizieren"),
    ("Delete", "Löschen"),
    ("Put Back", "Zurücklegen"),
    ("Convert to Project", "In Projekt umwandeln"),
    ("Rename", "Umbenennen"),
    ("Undo", "Rückgängig"),
    ("Redo", "Wiederholen (Redo)"),
    ("Empty Trash", "Papierkorb leeren"),
    ("Log Completed", "Erledigte ins Logbuch"),
    ("Delete List", "Liste löschen"),
    ("Move Up", "Nach oben"),
    ("Move Down", "Nach unten"),
    ("Quick Entry", "Schnelleingabe"),
    ("Show Today", "Heute anzeigen"),
    ("Clear", "Entfernen"),
    ("Clear reminder", "Erinnerung entfernen"),
    ("Remove deadline", "Frist entfernen"),
    ("Stop repeating", "Nicht mehr wiederholen"),
    ("No Project", "Kein Projekt"),
    ("Type a date: fri, 24.12., in 3 days, tomorrow 9:00", "Datum tippen: fr, 24.12., in 3 tagen, morgen 9:00"),
    ("Type to filter", "Tippen zum Filtern"),
    ("Type a date", "Datum tippen"),
    ("Search everything", "Alles durchsuchen"),
    ("every 2 weeks, after 3 days, until 2027-01-01", "every 2 weeks, after 3 days, until 2027-01-01"),
    ("Create tag", "Tag anlegen"),
    ("Daily", "Täglich"),
    ("Every weekday", "Jeden Werktag"),
    ("Weekly", "Wöchentlich"),
    ("Every 2 weeks", "Alle 2 Wochen"),
    ("Monthly", "Monatlich"),
    ("Yearly", "Jährlich"),
    ("Daily after completion", "Täglich nach Erledigung"),
    ("Weekly after completion", "Wöchentlich nach Erledigung"),
    ("Monthly after completion", "Monatlich nach Erledigung"),
    ("Next week", "Nächste Woche"),
    ("Nothing here", "Nichts zu tun"),
    ("New to-do", "Neues To-Do"),
    ("Later", "Später"),
    ("Overdue", "Überfällig"),
    ("left", "übrig"),
    ("Create a password", "Passwort festlegen"),
    ("Everything you add stays on this device, encrypted with this password. It cannot be recovered if you lose it.", "Alles bleibt auf diesem Gerät und wird mit diesem Passwort verschlüsselt. Geht es verloren, gibt es keinen Weg zurück."),
    ("Password", "Passwort"),
    ("Repeat password", "Passwort wiederholen"),
    ("Passwords do not match", "Die Passwörter stimmen nicht überein"),
    ("Create vault", "Tresor anlegen"),
    ("Unlock", "Entsperren"),
    ("Unlock with KeePassXC", "Mit KeePassXC entsperren"),
    ("gotodo is locked", "gotodo ist gesperrt"),
    ("Working…", "Einen Moment…"),
    ("KeePassXC is locked", "KeePassXC ist gesperrt"),
    ("KeePassXC is unlocked", "KeePassXC ist entsperrt"),
    ("KeePassXC not found", "KeePassXC nicht gefunden"),
    ("Another secret service is running, not KeePassXC", "Ein anderer Secret-Dienst läuft, nicht KeePassXC"),
    ("Wrong password", "Falsches Passwort"),
    ("General", "Allgemein"),
    ("Appearance", "Darstellung"),
    ("Automatic", "Automatisch"),
    ("Light", "Hell"),
    ("Dark", "Dunkel"),
    ("Text size", "Textgröße"),
    ("Reduce motion", "Animationen reduzieren"),
    ("Group Today by project", "Heute nach Projekt gruppieren"),
    ("Log completed items", "Erledigte protokollieren"),
    ("Immediately", "Sofort"),
    ("Manually", "Manuell"),
    ("Quick Entry adds to Today", "Schnelleingabe landet in Heute"),
    ("Security", "Sicherheit"),
    ("Auto-lock after", "Automatisch sperren nach"),
    ("minutes", "Minuten"),
    ("Never", "Nie"),
    ("Show titles in reminders", "Titel in Erinnerungen anzeigen"),
    ("Change password", "Passwort ändern"),
    ("Current password", "Aktuelles Passwort"),
    ("New password", "Neues Passwort"),
    ("Password changed", "Passwort geändert"),
    ("Connect KeePassXC", "KeePassXC verbinden"),
    ("Disconnect KeePassXC", "KeePassXC trennen"),
    ("KeePassXC connected", "KeePassXC verbunden"),
    ("In KeePassXC, enable Secret Service Integration and expose a group for this database.", "In KeePassXC die Secret-Service-Integration aktivieren und für diese Datenbank eine Gruppe freigeben."),
    ("Reminder", "Erinnerung"),
    ("A to-do is due", "Ein To-Do steht an"),
    ("Press Enter to save, Shift+Enter to add another, Esc to close", "Enter speichert, Shift+Enter für das nächste, Esc schließt"),
    ("Tab completes", "Tab ergänzt"),
    ("Could not save", "Speichern fehlgeschlagen"),
    ("Close", "Schließen"),
    ("Done", "Fertig"),
    ("Show", "Anzeigen"),
    ("Quick Find", "Schnellsuche"),
    ("Next item", "Nächster Eintrag"),
    ("Previous item", "Vorheriger Eintrag"),
    ("Select all", "Alles auswählen"),
    ("Open selection", "Auswahl öffnen"),
    ("Connected", "Verbunden"),
    ("Your to-dos are encrypted on this device.", "Deine To-Dos sind auf diesem Gerät verschlüsselt."),
    ("Enter your password to continue.", "Gib dein Passwort ein, um fortzufahren."),
    ("Use password", "Passwort verwenden"),
    ("Use this device", "Dieses Gerät verwenden"),
    ("Use KeePassXC", "KeePassXC verwenden"),
    ("Encrypted on this device", "Auf diesem Gerät verschlüsselt"),
    ("Type a date, optionally with a time: fri 17:00", "Datum eingeben, optional mit Uhrzeit: fr 17:00"),
    ("Notify before a timed deadline", "Vor einer Frist mit Uhrzeit benachrichtigen"),
    ("Off", "Aus"),
    ("Due in", "Fällig in"),
    ("Due now", "Jetzt fällig"),
    ("min", "Min."),
    ("h", "Std."),
    ("Welcome to gotodo", "Willkommen bei gotodo"),
    ("Your to-dos stay on this computer, encrypted. The key is sealed to this machine, so there is no password to type.", "Deine To-Dos bleiben verschlüsselt auf diesem Computer. Der Schlüssel ist an dieses Gerät gebunden, du musst kein Passwort tippen."),
    ("Get started", "Loslegen"),
    ("Use a password instead", "Stattdessen Passwort verwenden"),
    ("Use the device key instead", "Stattdessen Gerätschlüssel verwenden"),
    ("This computer could not open the device key", "Dieser Computer konnte den Gerätschlüssel nicht öffnen"),
    ("Device unlock is on", "Geräte-Entsperrung ist an"),
    ("Device unlock is off", "Geräte-Entsperrung ist aus"),
    ("Unlock with this device", "Mit diesem Gerät entsperren"),
    ("On. The key is sealed to this computer.", "An. Der Schlüssel ist an diesen Computer gebunden."),
    ("Off. A password or KeePassXC opens gotodo.", "Aus. Ein Passwort oder KeePassXC öffnet gotodo."),
    ("Turn off", "Ausschalten"),
    ("Turn on", "Einschalten"),
    ("Recovery password", "Wiederherstellungspasswort"),
    ("Set password", "Passwort setzen"),
    ("Password set", "Passwort gesetzt"),
    ("GNOME Keyring holds the Secret Service. Quit it (killall gnome-keyring-daemon), then turn Secret Service Integration off and on again in KeePassXC.", "GNOME Keyring belegt den Secret Service. Beende es (killall gnome-keyring-daemon), dann Secret-Service-Integration in KeePassXC aus- und wieder einschalten."),
    ("Zoom in", "Vergrößern"),
    ("Zoom out", "Verkleinern"),
    ("Reset zoom", "Zoom zurücksetzen"),
    ("Press Ctrl+N to add a to-do", "Strg+N legt ein neues To-Do an"),
    ("Manage", "Verwalten"),
    ("Set up", "Einrichten"),
    ("No password to type. gotodo opens whenever your KeePassXC database is unlocked.", "Kein Passwort tippen. gotodo öffnet sich, sobald deine KeePassXC-Datenbank entsperrt ist."),
    ("Turn on Secret Service in KeePassXC", "Secret Service in KeePassXC einschalten"),
    ("Tools › Settings › Secret Service Integration, then expose a group for this database.", "Extras › Einstellungen › Secret-Service-Integration, dann für diese Datenbank eine Gruppe freigeben."),
    ("Unlock your database", "Datenbank entsperren"),
    ("Open it the way you always do.", "So wie immer."),
    ("Connect", "Verbinden"),
    ("gotodo stores a random key in that group. It never leaves KeePassXC.", "gotodo legt einen zufälligen Schlüssel in dieser Gruppe ab. Er verlässt KeePassXC nie."),
    ("Done. Next time, gotodo opens by itself while KeePassXC is unlocked.", "Fertig. Ab jetzt öffnet sich gotodo von selbst, solange KeePassXC entsperrt ist."),
    ("Everything is ready. One click connects it.", "Alles bereit. Ein Klick verbindet es."),
    ("This page updates by itself while you set things up in KeePassXC.", "Diese Seite aktualisiert sich selbst, während du KeePassXC einrichtest."),
    ("Lock KeePassXC and gotodo locks with it. Your password keeps working either way.", "Sperrst du KeePassXC, sperrt sich gotodo mit. Dein Passwort funktioniert weiterhin."),
    ("Completed", "Erledigt"),
    ("to-dos completed", "To-Dos erledigt"),
    ("Canceled", "Abgebrochen"),
    ("to-dos canceled", "To-Dos abgebrochen"),
    ("Moved to Trash", "In den Papierkorb gelegt"),
    ("to-dos moved to Trash", "To-Dos in den Papierkorb gelegt"),
    ("Saved to", "Gespeichert unter"),
    ("Restored the copy from", "Wiederhergestellt ist die Kopie vom"),
    ("Unlock it the way you did then.", "Entsperre sie so wie damals."),
    ("Hide password", "Passwort verbergen"),
    ("Show password", "Passwort anzeigen"),
    ("Caps Lock is on", "Feststelltaste ist an"),
    ("Long enough", "Lang genug"),
    ("At least", "Mindestens"),
    ("characters", "Zeichen"),
    ("more", "weitere"),
    ("Passwords match", "Passwörter stimmen überein"),
    ("Unlocking…", "Wird entsperrt…"),
    ("Setting up…", "Wird eingerichtet…"),
    ("You locked gotodo, so this device cannot open it on its own. Unlock once with your password or KeePassXC.", "Du hast gotodo gesperrt, deshalb öffnet dieses Gerät es nicht allein. Entsperre einmal mit deinem Passwort oder KeePassXC."),
    ("Inbox is empty", "Der Eingang ist leer"),
    ("New to-dos land here until you give them a place.", "Neue To-Dos landen hier, bis du ihnen einen Platz gibst."),
    ("Nothing planned for today", "Heute ist nichts geplant"),
    ("Plan a to-do for today with Ctrl+T.", "Mit Strg+T planst du ein To-Do für heute."),
    ("Nothing coming up", "Nichts steht an"),
    ("To-dos with a date show up here, day by day.", "To-Dos mit Datum erscheinen hier, Tag für Tag."),
    ("Nothing in Anytime", "Nichts unter Jederzeit"),
    ("To-dos you can do whenever you like.", "To-Dos, die du erledigen kannst, wann du willst."),
    ("Nothing for someday", "Nichts für irgendwann"),
    ("Park ideas here that are not for now.", "Parke hier Ideen, die nicht für jetzt sind."),
    ("The Logbook is empty", "Das Logbuch ist leer"),
    ("Completed to-dos collect here.", "Erledigte To-Dos sammeln sich hier."),
    ("Trash is empty", "Der Papierkorb ist leer"),
    ("Deleted to-dos wait here until you empty it.", "Gelöschte To-Dos warten hier, bis du ihn leerst."),
    ("No deadlines", "Keine Fristen"),
    ("To-dos with a deadline show up here.", "To-Dos mit Frist erscheinen hier."),
    ("Nothing repeats", "Nichts wiederholt sich"),
    ("Make a to-do repeat with Ctrl+Shift+R.", "Mit Strg+Umschalt+R wiederholt sich ein To-Do."),
    ("Nothing here yet", "Noch nichts hier"),
    ("Add the first to-do to this list.", "Leg das erste To-Do in dieser Liste an."),
    ("new to-do", "neues To-Do"),
    ("Go to a list", "Zu einer Liste"),
    ("Open", "Öffnen"),
    ("Zoom", "Zoom"),
    ("Create", "Anlegen"),
    ("Schedule", "Planen"),
    ("Organize", "Ordnen"),
    ("Navigate", "Navigieren"),
    ("App", "App"),
    ("project", "Projekt"),
    ("tag", "Tag"),
    ("when", "wann"),
    ("deadline", "Frist"),
    ("completes", "ergänzt"),
    ("today", "heute"),
    ("overdue", "überfällig"),
    ("Move completed to the Logbook", "Erledigte ins Logbuch verschieben"),
    ("Otherwise new to-dos land in the Inbox.", "Sonst landen neue To-Dos im Eingang."),
    ("Lock after inactivity", "Sperren nach Inaktivität"),
    ("Show titles in notifications", "Titel in Benachrichtigungen zeigen"),
    ("Off: a notification only says that a to-do is due.", "Aus: Eine Benachrichtigung sagt nur, dass ein To-Do fällig ist."),
    ("Require password after locking", "Nach dem Sperren Passwort verlangen"),
    ("The device key stays unused until you unlock with your password or KeePassXC.", "Der Geräteschlüssel bleibt ungenutzt, bis du mit Passwort oder KeePassXC entsperrst."),
    ("Opens gotodo if this computer or KeePassXC cannot.", "Öffnet gotodo, falls dieser Computer oder KeePassXC es nicht kann."),
    ("Saves the encrypted file to your Documents folder.", "Speichert die verschlüsselte Datei in deinem Dokumente-Ordner."),
    ("Export a copy", "Kopie exportieren"),
    ("Export", "Exportieren"),
    ("Daily copies", "Tägliche Kopien"),
    ("The first copy is made the next time something changes.", "Die erste Kopie entsteht bei der nächsten Änderung."),
    ("Restore now", "Jetzt wiederherstellen"),
    ("Restore", "Wiederherstellen"),
    ("Locks gotodo and replaces your to-dos with this copy. Today's state is kept.", "Sperrt gotodo und ersetzt deine To-Dos durch diese Kopie. Der heutige Stand bleibt erhalten."),
    ("Lists", "Listen"),
    ("Backups", "Sicherungen"),
    ("gotodo keeps an encrypted copy from each of the last", "gotodo bewahrt eine verschlüsselte Kopie von jedem der letzten"),
    ("days. Changing the password starts over.", "Tage auf. Ein neues Passwort beginnt von vorn."),
    ("In 1 hour", "In 1 Stunde"),
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every English string the UI passes through `t` has a German entry.
    /// Finds direct `t("…")` calls and the literals handed to the helpers
    /// that call `t` themselves.
    #[test]
    fn every_ui_string_is_translated() {
        let helpers = [
            "t(", "setting_row(", "entry(", "group(", "link(", "auth_button(", "password_field(", "say(", "token(",
            "step(", "action(", "small_field(", "toggle(", "=> (", "(Icon::",
        ];
        // Key names stay as they are on the keyboard.
        let keys = ["Del", "Enter", "Esc", "Ctrl+N", "Ctrl+1–7", "Ctrl+± 0"];
        let mut missing = Vec::new();
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        let mut files = vec![std::path::PathBuf::from(root)];
        while let Some(path) = files.pop() {
            if path.is_dir() {
                files.extend(std::fs::read_dir(&path).unwrap().map(|e| e.unwrap().path()));
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") || path.ends_with("i18n.rs") || path.ends_with("tests.rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            // Unit tests sit at the end of a file; their strings are fixtures.
            let source = source.split("#[cfg(test)]\nmod tests").next().unwrap_or_default();
            // A call split over lines reads as one line from its opening parenthesis on.
            let mut joined = String::new();
            for line in source.lines() {
                joined.push_str(line.trim());
                joined.push(if line.trim_end().ends_with('(') { ' ' } else { '\n' });
            }
            for line in joined.lines() {
                let line = line.trim();
                if line.starts_with("//") {
                    continue;
                }
                let called = |h: &str| {
                    line.match_indices(h)
                        .filter(|(i, _)| !line[..*i].ends_with(|c: char| c.is_alphanumeric() || c == '_'))
                        .map(|(i, _)| i + h.len())
                        .next()
                };
                let Some(start) = helpers.iter().filter_map(|h| called(h)).min() else { continue };
                let mut rest = &line[start..];
                while let Some(open) = rest.find('"') {
                    let after = &rest[open + 1..];
                    let Some(close) = after.find('"') else { break };
                    let text = &after[..close];
                    rest = &after[close + 1..];
                    let wordy = text.chars().next().is_some_and(char::is_alphabetic);
                    let code = text.contains("::") || text.contains('{') || (!text.contains(' ') && text.chars().all(|c| c.is_lowercase()));
                    if wordy && !code && !keys.contains(&text) && !TABLE.iter().any(|(en, _)| *en == text) && !missing.contains(&text.to_owned()) {
                        missing.push(text.to_owned());
                    }
                }
            }
        }
        assert!(missing.is_empty(), "missing German for:\n{}", missing.join("\n"));
    }
}
