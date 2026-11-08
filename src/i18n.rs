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
];
