import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/navigation/navigator/goto_tab.dart';
import 'package:gotodo/views/tabs/look_up_tab/look_up_root.dart';

import '../../../framework/corporate_identity/goto_constants.dart';

class LookUpTab extends GotoTab {
  static const String root = "/";

  const LookUpTab({
    Key? key,
  }) : super(
          key: key,
          tabTag: GotoConstants.todosTabTag,
          routes: const {
            root: LookUpRoot(),
          },
        );
}
