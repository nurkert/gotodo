import 'package:flutter/material.dart';

class GoToNavigators {
  static final GoToNavigators _gotoNavigators = GoToNavigators._internal();

  factory GoToNavigators() {
    return _gotoNavigators;
  }

  GoToNavigators._internal();

  final Map<String, GlobalKey<NavigatorState>> navigators = {};

  GlobalKey<NavigatorState> initializeKey(String tag) {
    GlobalKey<NavigatorState> key = GlobalKey<NavigatorState>(
      debugLabel: tag,
    );
    navigators[tag] = key;
    return key;
  }

  GlobalKey<NavigatorState> loadKey(String tag) {
    return navigators[tag] ?? initializeKey(tag);
  }

  NavigatorState loadState(String tag) {
    return navigators[tag]!.currentState!;
  }
}
