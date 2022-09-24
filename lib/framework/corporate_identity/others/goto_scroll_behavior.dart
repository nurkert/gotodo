import 'package:flutter/material.dart';

/// a rather boring scrolling behaviour, which ensures that no overlay appears
/// on Android when you reach the end or the top of a scrollable list or try to
/// go beyond this end.
class GotoScrollBehavior extends ScrollBehavior {
  @override
  Widget buildOverscrollIndicator(
      BuildContext context, Widget child, ScrollableDetails details) {
    return child;
  }
}
