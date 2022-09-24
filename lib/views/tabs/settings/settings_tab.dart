import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';
import 'package:gotodo/framework/corporate_identity/widgets/navigation/navigator/goto_tab.dart';
import 'package:gotodo/views/tabs/settings/settings_general.dart';
import 'package:gotodo/views/tabs/settings/settings_root.dart';

class SettingsTab extends GotoTab {
  static const String root = "/";
  static const String general = "/general/";

  const SettingsTab({
    Key? key,
  }) : super(
          key: key,
          tabTag: GotoConstants.settingsTabTag,
          routes: const {
            root: SettingsRoot(),
            general: SettingsGeneral(),
          },
        );
}
