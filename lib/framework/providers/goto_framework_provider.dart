import 'package:flutter/material.dart';

import '../language/goto_dictionary.dart';
import '../language/goto_language.dart';

class GotoFrameworkProvider extends ChangeNotifier {
  // TODO: Theme Einstellung auslesen
  ThemeMode themeMode = ThemeMode.system;

  bool get isDarkTheme => themeMode == ThemeMode.dark;

  void setDarkTheme(bool isDarkTheme) {
    // TODO: Theme Einstellung speichern
    themeMode = isDarkTheme ? ThemeMode.dark : ThemeMode.light;
    notifyListeners();
  }

  // TODO: Sprache auslesen
  GotoLanguage dictionary = GotoTexts.german;

  GotoLanguage get texts => dictionary;

  void switchLanguage(GotoLanguage language) {
    dictionary = language;
    notifyListeners();
  }
}
