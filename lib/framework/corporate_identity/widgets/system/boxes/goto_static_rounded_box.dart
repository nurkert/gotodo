import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';

class GotoRoundedBox extends StatelessWidget {
  const GotoRoundedBox({
    required this.height,
    required this.child,
    this.width,
    this.color,
    Key? key,
  }) : super(key: key);

  /// Height of the box
  final double height;

  /// Width of the box
  final double? width;

  /// Content of the box
  final Widget child;

  /// special dye for the rounded box
  final Color? color;

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: color ?? Theme.of(context).splashColor,
        borderRadius: const BorderRadius.all(
          Radius.circular(GotoConstants.circularBorderRadius),
        ),
      ),
      height: height,
      width: width,
      child: child,
    );
  }
}
