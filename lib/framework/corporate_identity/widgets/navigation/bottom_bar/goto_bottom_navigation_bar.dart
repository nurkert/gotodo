import 'package:flutter/material.dart';

import '../../../../goto_central.dart';
import '../../../goto_constants.dart';

class GoToBottomNavigation extends StatefulWidget {
  const GoToBottomNavigation({
    required this.tabs,
    required this.onChanged,
    this.defaultSelected,
    Key? key,
  }) : super(key: key);

  // A map consisting of the icons and the associated subtitles representing the tabs.
  final Map<IconData, String> tabs;

  /// function that is called when one of the icons is clicked.
  ///
  /// the [index] represents the position of the clicked icon
  final void Function(int index) onChanged;

  /// the index of the item that should be selected when creating the widget
  final int? defaultSelected;

  @override
  _GoToBottomNavigationState createState() => _GoToBottomNavigationState();
}

class _GoToBottomNavigationState extends State<GoToBottomNavigation> {
  /// index of the currently selected item
  // late int _selected;

  /// the size of the tab icons that are not currently selected should have
  final double _unselectedSize = 24;

  @override
  void didChangeDependencies() {
    // initialise which tab is to be displayed at the beginning
    // _selected = widget.defaultSelected ?? 0;
    super.didChangeDependencies();
  }

  @override
  Widget build(BuildContext context) {
    double mediaWidth = MediaQuery.of(context).size.width;

    // the maximum space that the icons with text may fill
    // double widthPerIcon = (mediaWidth * 0.9 - GotoConstants.actionButtonSize) /
    //     widget.tabs.entries.length;

    return Padding(
      padding: EdgeInsets.only(
        top: 7.5,
        left: mediaWidth * 0.05,
        right: mediaWidth * 0.05 + GotoConstants.actionButtonSize,
      ),
      child: SizedBox(
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            for (int index = 0; index < widget.tabs.length; index++)
              GestureDetector(
                onTap: () => widget.onChanged(index),
                child: TweenAnimationBuilder<double>(
                  tween: Tween<double>(
                    begin: _unselectedSize,
                    end: index == GotoCentral().selectedTabIndex
                        ? GotoConstants.iconSize
                        : _unselectedSize,
                  ),
                  duration: const Duration(milliseconds: 100),
                  builder: (
                    BuildContext context,
                    double size,
                    Widget? child,
                  ) {
                    return Container(
                      // enlarges the hitbox
                      color: Colors.transparent,
                      child: Column(
                        children: [
                          Icon(
                            widget.tabs.keys.elementAt(index),
                            size: size,
                            color: index == GotoCentral().selectedTabIndex
                                ? Theme.of(context).primaryColor
                                : Theme.of(context)
                                    .appBarTheme
                                    .toolbarTextStyle!
                                    .backgroundColor!,
                          ),
                          Text(
                            widget.tabs.values.elementAt(index),
                            style: TextStyle(
                              fontSize: size * 0.4,
                              color: index == GotoCentral().selectedTabIndex
                                  ? Theme.of(context).primaryColor
                                  : Theme.of(context)
                                      .appBarTheme
                                      .toolbarTextStyle!
                                      .backgroundColor!,
                            ),
                          ),
                        ],
                      ),
                    );
                  },
                ),
              ),
          ],
        ),
      ),
    );
  }
}
