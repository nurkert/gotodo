import 'package:flutter/material.dart';

class GotoCentral {
  static final GotoCentral _centralised = GotoCentral._internal();

  factory GotoCentral() {
    return _centralised;
  }

  GotoCentral._internal();

  int selectedTabIndex = 0;

  ScrollController settingsRootScrollController = ScrollController();
  ScrollController todosRootScrollController = ScrollController();
}
