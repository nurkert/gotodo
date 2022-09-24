import 'package:gotodo/framework/corporate_identity/widgets/navigation/navigator/goto_tab.dart';
import 'package:flutter/material.dart';
import 'package:gotodo/views/tabs/todos/todos_root.dart';

import '../../../framework/corporate_identity/goto_constants.dart';

class TodosTab extends GotoTab {
  static const String root = "/";

  TodosTab({
    Key? key,
  }) : super(
          key: key,
          tabTag: GotoConstants.todosTabTag,
          routes: {
            root: TodosRoot(),
          },
        );
}
