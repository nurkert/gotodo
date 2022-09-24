import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';

// ignore: must_be_immutable
class GotoAnimatedRoundedBox extends StatelessWidget {
  const GotoAnimatedRoundedBox({
    required this.height,
    this.child,
    this.duration,
    this.color,
    this.curve,
    Key? key,
  }) : super(key: key);

  final double height;

  final Widget? child;

  final Color? color;

  final Duration? duration;

  final Curve? curve;

  @override
  Widget build(BuildContext context) {
    double mediaWidth = MediaQuery.of(context).size.width;
    return SizedBox(
      // width: mediaWidth * 0.9,
      child: AnimatedContainer(
        duration: duration ?? const Duration(milliseconds: 200),
        curve: curve ?? Curves.ease,
        decoration: BoxDecoration(
          color: color ?? Theme.of(context).splashColor,
          borderRadius: const BorderRadius.all(
            Radius.circular(GotoConstants.circularBorderRadius),
          ),
          // boxShadow: [
          //   if (shadow)
          //     BoxShadow(
          //       color: Colors.white.withOpacity(0.1),
          //       spreadRadius: 5,
          //       blurRadius: 15,
          //     ),
          // ],
        ),
        height: height,
        child: ClipRRect(
            borderRadius: const BorderRadius.all(
              Radius.circular(
                GotoConstants.circularBorderRadius,
              ),
            ),
            child: child),
      ),
    );
  }
}
