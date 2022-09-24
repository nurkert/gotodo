import 'package:flutter/material.dart';
import 'package:gotodo/framework/language/dictionary_categories/add_item_dictionary.dart';
import 'package:gotodo/framework/language/dictionary_categories/general_dictionary.dart';
import 'package:gotodo/framework/language/dictionary_categories/settings_dictionary.dart';
import 'package:gotodo/framework/language/dictionary_categories/titles_dictionary.dart';
import 'package:gotodo/framework/language/goto_language.dart';
import 'package:provider/provider.dart';

import '../providers/goto_framework_provider.dart';
import 'dictionary_categories/bottom_navigation_dictionary.dart';

class GotoDictionary {
  /// Returns the currently configured dictionary
  static GotoLanguage of(BuildContext context, [bool listen = false]) {
    final frameworkProvider =
        Provider.of<GotoFrameworkProvider>(context, listen: listen);
    return frameworkProvider.dictionary;
  }
}

/// A class that initialises the various text modules of the app with the
/// corresponding strings of a language.
class GotoTexts {
  static const german = GotoLanguage(
    titles: TitlesDictionary(
      todos: 'Aufgaben',
      shoppingList: 'Einkaufsliste',
      lookUp: 'Suchen',
      notes: 'Notizen',
      settings: 'Einstellungen',
      settingsGeneral: 'Allgemein',
    ),
    bottomNavBar: BottomNavigationDictionary(
      todos: 'Aufgaben',
      shopping: 'Einkaufsliste',
      lookUp: 'Suchen',
      notes: 'Notizen',
      settings: 'Einstellungen',
    ),
    general: GeneralDictionary(
      appDescription: 'Die intelligente Lösung für deine ToDo-App.',
      toBeContinued: 'Inhalt folgt...',
    ),
    settings: SettingsDictionary(
      darkMode: 'Dunkles Design',
      language: 'Sprache',
    ),
    addItem: AddItemDictionary(
      todo: "Aufgabe",
      entry: "Eintrag",
      newTodo: "Neue Aufgabe",
      newEntry: "Neuer Eintrag",
      notes: "Notizen",
    ),
  );

  static const english = GotoLanguage(
    titles: TitlesDictionary(
      todos: 'ToDo\'s',
      shoppingList: 'Shopping list',
      lookUp: 'Look up',
      notes: 'Notes',
      settings: 'Settings',
      settingsGeneral: 'General',
    ),
    bottomNavBar: BottomNavigationDictionary(
      todos: 'ToDo\'s',
      shopping: 'Shopping list',
      lookUp: 'Look up',
      notes: 'Notes',
      settings: 'Settings',
    ),
    general: GeneralDictionary(
      appDescription: 'The smart solution for your ToDo app.',
      toBeContinued: 'To be continued...',
    ),
    settings: SettingsDictionary(
      darkMode: 'Dark mode',
      language: 'Language',
    ),
    addItem: AddItemDictionary(
      todo: "Todo",
      entry: "Entry",
      newTodo: "New todo",
      newEntry: "New entry",
      notes: "Notes",
    ),
  );
}
