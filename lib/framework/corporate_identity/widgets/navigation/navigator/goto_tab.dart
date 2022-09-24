import 'package:flutter/material.dart';

import 'goto_tab_navigator.dart';

abstract class GotoTab extends StatelessWidget {
  const GotoTab({
    Key? key,
    required this.tabTag,
    required this.routes,
  }) : super(key: key);

  final String tabTag;

  final Map<String, Widget> routes;

  @override
  Widget build(BuildContext context) {
    return GotoTabNavigator(
      tag: tabTag,
      routes: routes,
    );
  }

  /// adds a key and a route to the tab
  ///

}
