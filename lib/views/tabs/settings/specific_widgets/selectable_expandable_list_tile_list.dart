import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/boxes/goto_animated_rounded_box.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/list_tiles/goto_list_tile.dart';

import '../../../../framework/corporate_identity/goto_constants.dart';

// ignore: must_be_immutable
class SelectableExpandableListTileList extends StatefulWidget {
  SelectableExpandableListTileList({
    Key? key,
    required this.text,
    required this.selected,
    required this.onSelected,
    required this.options,
  }) : super(key: key);

  /// few words describing the option to be set
  final String text;

  /// function that is called when a new selection is made
  ///
  /// The passed [int] is the index of the newly made selection.
  final void Function(int) onSelected;

  /// a list of options that the user can choose from
  final List<String> options;

  /// the index of the currently selected option
  int selected;

  @override
  _SelectableExpandableListTileListState createState() =>
      _SelectableExpandableListTileListState();
}

class _SelectableExpandableListTileListState
    extends State<SelectableExpandableListTileList>
    with SingleTickerProviderStateMixin {
  /// Is the widget currently extended to make other options selectable?
  bool _expanded = false;

  /// how long should the selection animation take
  final _animationDuration = const Duration(milliseconds: 200);

  late AnimationController _controller;

  @override
  void initState() {
    _controller = AnimationController(
      vsync: this,
      duration: _animationDuration,
      upperBound: 0.5,
    );
    super.initState();
  }

  @override
  void dispose() {
    super.dispose();
    _controller.dispose();
  }

  @override
  Widget build(BuildContext context) {
    double mediaWidth = MediaQuery.of(context).size.width;

    /// amount of list tile widgets to be displayed
    int amount = _expanded ? widget.options.length + 1 : 1;

    /// the adaptive size of the entire widget to the number of list tile widgets to be displayed.
    double boxHeight = amount * GotoConstants.listTileHeight;

    /// arrow on the right indicating whether the widget will expand or shrink next
    ///
    /// during the extend animation, the arrow rotates in the same time.
    Widget arrow = RotationTransition(
      turns: Tween(begin: 0.0, end: 1.0).animate(_controller),
      child: Transform.rotate(
        angle: -math.pi / 2,
        child: const Icon(
          Icons.arrow_back_ios_new_rounded,
          size: GotoConstants.iconSize / 2,
        ),
      ),
    );

    /// a list of all list tile widgets to be displayed
    List<Widget> children = [
      /// The always displayed list tile, which describes the widget for the user in a general
      GestureDetector(
        child: GotoListTile(
          text: widget.text,
          icon: Row(
            mainAxisAlignment: MainAxisAlignment.end,
            children: [
              // if widget is extended, do not show selection
              AnimatedSwitcher(
                duration: _animationDuration,
                child: _expanded
                    ? Container()
                    : Text(
                        // Text of the current selection
                        widget.options[widget.selected],
                        style: TextStyle(
                          color: Theme.of(context)
                              .appBarTheme
                              .toolbarTextStyle!
                              .backgroundColor!,
                        ),
                      ),
              ),
              SizedBox(width: mediaWidth * 0.02),
              arrow,
            ],
          ),
        ),
        onTap: () {
          setState(() {
            if (_expanded) {
              /// makes [arrow] widget turn upwards
              _controller.reverse(from: 0.5);
            } else {
              /// makes [arrow] widget turn downwards
              _controller.forward(from: 0.0);
            }
            // Causes the widget to shrink or extend.
            _expanded = !_expanded;
          });
        },
      ),
    ];

    if (_expanded) {
      // if widget is extended, show all options
      for (int i = 0; i < widget.options.length; i++) {
        children.add(const Divider(height: 0)); // separating line
        children.add(
          GestureDetector(
            onTap: () {
              // mark clicked option as new selection
              setState(() {
                widget.selected = i;
                widget.onSelected(i);
              });
            },
            child: GotoListTile(
              text: widget.options[i], // Descriptive text of the option
              icon: widget.selected == i // tick behind option if it is selected
                  ? const Icon(
                      Icons.check_rounded,
                      size: GotoConstants.iconSize / 2,
                    )
                  : null,
            ),
          ),
        );
      }
    }

    return GotoAnimatedRoundedBox(
      height: boxHeight,
      duration: _animationDuration,
      child: SingleChildScrollView(
          child: Column(
        children: children,
      )),
    );
  }
}
