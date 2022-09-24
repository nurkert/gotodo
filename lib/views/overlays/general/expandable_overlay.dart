import 'dart:ui';

import 'package:flutter/material.dart';

import '../../../framework/corporate_identity/goto_constants.dart';
import '../../../framework/corporate_identity/widgets/system/boxes/goto_animated_rounded_box.dart';
import '../../../framework/corporate_identity/widgets/system/buttons/goto_expandable_button.dart';
import '../../../framework/corporate_identity/widgets/system/buttons/goto_icon_button.dart';
import '../../../framework/language/goto_dictionary.dart';
import 'blurred_bar.dart';

// ignore: must_be_immutable
class ExpandableOverlay extends StatefulWidget {
  ExpandableOverlay({
    required this.child,
    required this.onExpand,
    this.expanded = false,
    Key? key,
  }) : super(key: key);

  final Widget child;

  /// The function to be called when the overlay is expanded
  final void Function(bool) onExpand;

  /// Whether the overlay is expanded or not
  bool expanded;

  @override
  _ExpandableOverlayState createState() => _ExpandableOverlayState();
}

class _ExpandableOverlayState extends State<ExpandableOverlay> {
  final BorderRadius _borderRadius = const BorderRadius.all(
    Radius.circular(GotoConstants.circularBorderRadius),
  );

  final Duration _animationDuration = const Duration(milliseconds: 350);

  @override
  Widget build(BuildContext context) {
    final mediaSize = MediaQuery.of(context).size;
    final dictionary = GotoDictionary.of(context);

    final barHeight = mediaSize.width * 0.1;

    final header = BlurredBar(
      height: barHeight,
      child: Stack(
        children: [
          Center(
            child: Text(
              dictionary.addItem.newTodo,
              style: Theme.of(context).textTheme.headline6!,
            ),
          ),
          Align(
            alignment: Alignment.centerLeft,
            child: Padding(
              padding: EdgeInsets.only(left: mediaSize.width * 0.04),
              child: GotoIconButton(
                onTap: () {
                  closeKeybaord();
                  Navigator.of(context).pop();
                },
                icon: Icons.close_rounded,
              ),
            ),
          ),
        ],
      ),
    );

    final footer = BlurredBar(
      height: barHeight,
      child: GotoExpandableButton(
        expanded: widget.expanded,
        onExpand: (expanded) => widget.onExpand(expanded),
      ),
    );

    double height =
        widget.expanded ? mediaSize.height * 0.45 : mediaSize.height * 0.23;

    return BackdropFilter(
      filter: ImageFilter.blur(
        sigmaX: GotoConstants.overlayBackgroundBlureness,
        sigmaY: GotoConstants.overlayBackgroundBlureness,
      ),
      child: IntrinsicHeight(
        child: Padding(
          padding: EdgeInsets.only(
            bottom: MediaQuery.of(context).viewInsets.bottom,
          ),
          child: Center(
            child: Padding(
              padding: EdgeInsets.symmetric(horizontal: mediaSize.width * 0.05),
              child: Hero(
                tag: GotoConstants.addItemHeroTag,
                createRectTween: GotoConstants.linearRectTween,
                child: Material(
                  // color: Colors.transparent,
                  shape: RoundedRectangleBorder(borderRadius: _borderRadius),
                  child: ClipRRect(
                    borderRadius: _borderRadius,
                    child: BackdropFilter(
                      filter: ImageFilter.blur(sigmaX: 0, sigmaY: 0),
                      child: GestureDetector(
                        onTap: () => closeKeybaord(),
                        child: GotoAnimatedRoundedBox(
                          duration: _animationDuration,
                          height: height,
                          color: Theme.of(context).dialogBackgroundColor,
                          child: Stack(
                            children: [
                              SingleChildScrollView(
                                child: Padding(
                                  padding: EdgeInsets.symmetric(
                                    vertical: barHeight + 5,
                                  ),
                                  child: widget.child,
                                ),
                              ),
                              Align(
                                alignment: Alignment.topCenter,
                                child: header,
                              ),
                              Align(
                                alignment: Alignment.bottomCenter,
                                child: footer,
                              ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  void closeKeybaord() => FocusScope.of(context).requestFocus(FocusNode());
}
