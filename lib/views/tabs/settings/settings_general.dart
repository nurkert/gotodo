import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/app_bar/goto_static_app_bar_template.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/list_tiles/goto_list_space.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/list_tiles/goto_switch_list_tile.dart';
import 'package:gotodo/framework/language/goto_language.dart';
import 'package:gotodo/views/tabs/settings/specific_widgets/selectable_expandable_list_tile_list.dart';
import 'package:provider/provider.dart';

import '../../../framework/corporate_identity/widgets/system/list_tiles/goto_list_tile_collection.dart';
import '../../../framework/language/goto_dictionary.dart';
import '../../../framework/providers/goto_framework_provider.dart';

class SettingsGeneral extends StatelessWidget {
  const SettingsGeneral({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final frameworkProvider = Provider.of<GotoFrameworkProvider>(context);
    final dictionary = GotoDictionary.of(context);

    /**
     * Settings for whether to use a dark or light design
     */

    final darkModeOption = GotoListTileCollection(
      listTiles: [
        GotoSwitchListTile(
          text: dictionary.settings.darkMode,
          value: frameworkProvider.isDarkTheme,
          onChanged: (value) {
            frameworkProvider.setDarkTheme(
              value,
            );
          },
        ),
      ],
    );

    /**
     * Settings about the language of the app
    */

    Map<String, GotoLanguage> languages = {
      "Deutsch": GotoTexts.german,
      "English": GotoTexts.english,
    };

    final selectLanguageOption = SelectableExpandableListTileList(
      text: dictionary.settings.language,
      selected: languages.values.toList().indexOf(dictionary),
      onSelected: (index) {
        GotoLanguage newLanguage = languages.values.elementAt(index);
        if (frameworkProvider.dictionary != newLanguage) {
          frameworkProvider.switchLanguage(newLanguage);
        }
      },
      options: languages.keys.toList(),
    );

    return GotoStaticAppbarTemplate(
      title: dictionary.titles.settingsGeneral,
      body: ListView(
        children: [
          const GotoListSpace(),
          darkModeOption,
          const GotoListSpace(),
          selectLanguageOption,
        ],
      ),
    );
  }
}
