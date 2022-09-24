import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/navigation/navigator/page_route/linear_rect_tween.dart';

class GotoConstants {
  /// the standardised size of the floating action button
  static const double actionButtonSize = 60;

  /// the standardised size of an icon
  static const double iconSize = 27.5;

  static const double circularBorderRadius = 10.0;

  static const double listTileHeight = 40;

  static const double appBarBlureness = 4;
  static const double appBarHeight = 60;

  static const double overlayBackgroundBlureness = 2;
  /*
  * Todo Tags of Navigators
  */
  static const String todosTabTag = "todos";
  static const String shoppingTabTag = "shopping";
  static const String notesTabTag = "notes";
  static const String settingsTabTag = "settings";
  static const String lookUpTabTag = "look_up";

  /// Hero Tag of add ToDo button and widget
  static const String addItemHeroTag = "add_to_do";

  /// linear Hero animation
  static Tween<Rect?> linearRectTween(Rect? begin, Rect? end) {
    return LinearRectTween(begin: begin!, end: end!);
  }
}
