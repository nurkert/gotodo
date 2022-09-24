import 'package:gotodo/framework/language/dictionary_categories/general_dictionary.dart';

import 'dictionary_categories/add_item_dictionary.dart';
import 'dictionary_categories/bottom_navigation_dictionary.dart';
import 'dictionary_categories/settings_dictionary.dart';
import 'dictionary_categories/titles_dictionary.dart';

class GotoLanguage {
  const GotoLanguage({
    required this.titles,
    required this.general,
    required this.settings,
    required this.bottomNavBar,
    required this.addItem,
  });

  final GeneralDictionary general;
  final TitlesDictionary titles;
  final BottomNavigationDictionary bottomNavBar;
  final SettingsDictionary settings;
  final AddItemDictionary addItem;
}
