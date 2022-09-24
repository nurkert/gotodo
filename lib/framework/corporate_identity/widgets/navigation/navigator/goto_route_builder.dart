import 'package:flutter/material.dart';

class GotoRouteBuilder {
  GotoRouteBuilder(this.routes);

  /// The map of routes to be used by the navigator
  final Map<String, Widget> routes;

  Map<String, Widget Function(BuildContext)> build(BuildContext context) {
    return {
      for (int index = 0; index < routes.entries.length; index++)
        routes.keys.elementAt(index): (context) =>
            routes.values.elementAt(index)
    };
  }
}
