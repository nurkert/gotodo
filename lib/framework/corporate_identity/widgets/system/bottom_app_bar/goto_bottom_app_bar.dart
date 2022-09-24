import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:gotodo/framework/language/goto_dictionary.dart';
import 'package:provider/provider.dart';

import '../../../../goto_central.dart';
import '../../../../providers/goto_framework_provider.dart';
import '../../../goto_constants.dart';
import '../../navigation/bottom_bar/goto_bottom_navigation_bar.dart';
import '../../navigation/navigator/goto_navigators.dart';
import '../../navigation/navigator/goto_tab.dart';

class GotoBottomAppBar extends StatefulWidget {
  const GotoBottomAppBar({
    required this.tabs,
    required this.pageController,
    Key? key,
  }) : super(key: key);

  final PageController pageController;
  final List<GotoTab> tabs;

  @override
  GotoBottomAppBarState createState() => GotoBottomAppBarState();
}

class GotoBottomAppBarState extends State<GotoBottomAppBar> {
  @override
  Widget build(BuildContext context) {
    final dictionary = GotoDictionary.of(context, true);
    return BottomAppBar(
      elevation: 0,
      clipBehavior: Clip.hardEdge,
      notchMargin: 10,
      color: Theme.of(context).appBarTheme.backgroundColor,
      shape: const CircularNotchedRectangle(),
      child: BackdropFilter(
        filter: ImageFilter.blur(
          sigmaX: GotoConstants.appBarBlureness,
          sigmaY: GotoConstants.appBarBlureness,
        ),
        child: SizedBox(
          height: 55.0,
          child: GoToBottomNavigation(
            defaultSelected: GotoCentral().selectedTabIndex,
            tabs: {
              Icons.list_alt_rounded: dictionary.bottomNavBar.todos,
              Icons.shopping_cart_outlined: dictionary.bottomNavBar.shopping,
              // Icons.search_rounded: dictionary.bottomNavBar.lookUp,
              Icons.note_alt_outlined: dictionary.bottomNavBar.notes,
              Icons.settings_rounded: dictionary.bottomNavBar.settings,
            },
            onChanged: (newTabIndex) async {
              if (GotoCentral().selectedTabIndex == newTabIndex) {
                final tag = widget.tabs[newTabIndex].tabTag;
                final state = GoToNavigators().loadState(tag);
                state.popUntil((route) => route.isFirst);
              } else {
                switchTab(newTabIndex);
              }
            },
          ),
        ),
      ),
    );
  }

  void switchTab(int newTabIndex) {
    setState(() {
      GotoCentral().selectedTabIndex = newTabIndex;
      widget.pageController.animateToPage(
        newTabIndex,
        duration: const Duration(milliseconds: 500),
        curve: Curves.ease,
      );
    });
  }
}
