import 'package:flutter/material.dart';
import 'package:gotodo/views/overlays/add_item/add_todo/priority_settings/goto_widget_selector.dart';

class PriorityOptionsSelectior extends StatelessWidget {
  const PriorityOptionsSelectior({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GotoWidgetSelector(
      color: Theme.of(context).splashColor,
      onChanged: (index) {},
      selectable: {
        for (int i = 0; i < 3; i++)
          "$i. Hero-Tag": Container(
            height: 10,
            width: 10,
          ),
      },
    );
  }
}
