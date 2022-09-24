import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/navigation/navigator/goto_navigators.dart';
import 'package:gotodo/views/tabs/notes/notes_tab.dart';
import 'package:gotodo/views/tabs/settings/settings_tab.dart';

import '../views/tabs/shopping/shopping_tab.dart';
import '../views/tabs/todos/todos_tab.dart';
import 'corporate_identity/widgets/navigation/navigator/goto_tab.dart';
import 'corporate_identity/widgets/system/bottom_app_bar/goto_bottom_app_bar.dart';
import 'corporate_identity/widgets/system/bottom_app_bar/goto_floating_action_button.dart';
import 'goto_central.dart';

class GoToHome extends StatefulWidget {
  /// the home or main widget of the app
  const GoToHome({
    // this.initialTab = 0,
    Key? key,
  }) : super(key: key);

  // final int initialTab;

  @override
  State<GoToHome> createState() => _GoToHomeState();
}

class _GoToHomeState extends State<GoToHome> {
  // int? _selectedTabIndex;

  PageController? _pageController;

  @override
  void didChangeDependencies() {
    _pageController = PageController(
      initialPage: GotoCentral().selectedTabIndex,
      keepPage: false,
    );
    super.didChangeDependencies();
  }

  final List<GotoTab> tabs = [
    TodosTab(),
    const ShoppingTab(),
    // LookUpTab(),
    const NotesTab(),
    const SettingsTab(),
  ];

  @override
  Widget build(BuildContext context) {
    return WillPopScope(
      onWillPop: () async {
        final int current = GotoCentral().selectedTabIndex;

        final String tag = tabs.elementAt(current).tabTag;

        if (!await GoToNavigators().loadState(tag).maybePop()) {
          setState(() {
            GotoCentral().selectedTabIndex = 0;
            _pageController!.animateToPage(
              0,
              duration: const Duration(milliseconds: 500),
              curve: Curves.ease,
            );
          });
        } else {
          GoToNavigators().loadState(tag).popUntil((route) => route.isFirst);
        }
        return false;
      },
      child: Scaffold(
        body: PageView(
          children: tabs,
          controller: _pageController,
          physics: const NeverScrollableScrollPhysics(),
        ),
        extendBody: true,
        bottomNavigationBar: GotoBottomAppBar(
          tabs: tabs,
          pageController: _pageController!,
        ),
        floatingActionButton: const GoToFloatingActionButton(),
        floatingActionButtonLocation:
            FloatingActionButtonLocation.miniEndDocked,
        backgroundColor: Theme.of(context).backgroundColor,
      ),
    );
  }
}
