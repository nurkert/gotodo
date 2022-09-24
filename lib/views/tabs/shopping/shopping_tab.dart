import 'package:flutter/material.dart';
import 'package:gotodo/views/tabs/shopping/shopping_root.dart';

import '../../../framework/corporate_identity/goto_constants.dart';
import '../../../framework/corporate_identity/widgets/navigation/navigator/goto_tab.dart';

class ShoppingTab extends GotoTab {
  static const String root = "/";

  const ShoppingTab({
    Key? key,
  }) : super(
          key: key,
          tabTag: GotoConstants.shoppingTabTag,
          routes: const {
            root: ShoppingRoot(),
          },
        );
}
