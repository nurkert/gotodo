import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/app_bar/goto_sliver_app_bar_template.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/list_tiles/goto_forward_list_tile.dart';
import 'package:gotodo/views/tabs/settings/settings_tab.dart';

import '../../../framework/corporate_identity/widgets/system/list_tiles/goto_list_tile_collection.dart';
import '../../../framework/language/goto_dictionary.dart';

class SettingsRoot extends StatefulWidget {
  const SettingsRoot({Key? key}) : super(key: key);

  @override
  State<SettingsRoot> createState() => _SettingsRootState();
}

class _SettingsRootState extends State<SettingsRoot> {
  @override
  Widget build(BuildContext context) {
    final dictionary = GotoDictionary.of(context);

    return GotoSliverAppbarTemplate(
      title: dictionary.titles.settings,
      body: ListView(
        physics: const BouncingScrollPhysics(),
        children: [
          GotoListTileCollection(
            listTiles: [
              GotoForwardListTile(
                text: dictionary.titles.settingsGeneral,
                routeName: SettingsTab.general,
              ),
            ],
          ),
        ],
      ),
    );
  }
}
