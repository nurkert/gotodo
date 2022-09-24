import 'package:flutter/material.dart';

import 'goto_icon_button.dart';

// ignore: must_be_immutable
class GotoExpandableButton extends StatefulWidget {
  GotoExpandableButton({
    required this.onExpand,
    required this.expanded,
    this.animationDuration = const Duration(milliseconds: 350),
    Key? key,
  }) : super(key: key);

  bool expanded;

  final void Function(bool) onExpand;

  /// The duration of the animation
  final Duration animationDuration;

  @override
  _GotoExpandableButtonState createState() => _GotoExpandableButtonState();
}

class _GotoExpandableButtonState extends State<GotoExpandableButton>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    _controller = AnimationController(
      vsync: this,
      duration: widget.animationDuration,
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
    return RotationTransition(
      turns: Tween(
        begin: 0.0,
        end: 1.0,
      ).animate(_controller),
      child: GotoIconButton(
        onTap: () {
          widget.expanded = !widget.expanded;
          if (widget.expanded) {
            _controller.forward(from: 0.0);
          } else {
            _controller.reverse(from: 0.5);
          }
          widget.onExpand(widget.expanded);
        },
        icon: Icons.keyboard_arrow_down_rounded,
      ),
    );
  }
}
