import 'package:flutter/material.dart';

import '../../../framework/corporate_identity/widgets/system/app_bar/goto_sliver_app_bar_template.dart';
import '../../../framework/corporate_identity/widgets/system/list_tiles/goto_list_tile_collection.dart';
import '../../../framework/language/goto_dictionary.dart';

class ShoppingRoot extends StatelessWidget {
  const ShoppingRoot({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    final dictionary = GotoDictionary.of(context);

    return GotoSliverAppbarTemplate(
      title: dictionary.titles.shoppingList,
      icon: MapEntry(Icons.menu_rounded, () {}),
      body: ListView(
        physics: const BouncingScrollPhysics(),
        children: const [
          GotoListTileCollection(
            listTiles: [],
          ),
        ],
      ),
    );
  }
}
