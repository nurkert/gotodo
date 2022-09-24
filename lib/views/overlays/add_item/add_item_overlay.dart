import 'package:flutter/material.dart';
import 'package:gotodo/views/overlays/add_item/add_todo/add_todo_dialog.dart';
import 'package:gotodo/views/overlays/general/expandable_overlay.dart';

class AddItemOverlay extends StatefulWidget {
  const AddItemOverlay({Key? key}) : super(key: key);

  @override
  _AddItemOverlayState createState() => _AddItemOverlayState();
}

class _AddItemOverlayState extends State<AddItemOverlay> {
  PageController _pageController = PageController();

  void closeKeybaord() => FocusScope.of(context).requestFocus(FocusNode());

  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    return ExpandableOverlay(
      expanded: _expanded,
      onExpand: (expanded) {
        setState(() {
          _expanded = expanded;
        });
      },
      child: AddTodoDialog(isExtended: _expanded),
    );
  }
}
