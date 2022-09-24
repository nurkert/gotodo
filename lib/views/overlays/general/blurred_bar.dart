import 'dart:ui';

import 'package:flutter/material.dart';

import '../../../framework/corporate_identity/goto_constants.dart';

class BlurredBar extends StatelessWidget {
  const BlurredBar({
    required this.child,
    required this.height,
    Key? key,
  }) : super(key: key);

  final Widget child;

  /// The height of the bar
  final double height;

  @override
  Widget build(BuildContext context) {
    final double mediaWidth = MediaQuery.of(context).size.width;
    return Container(
      color: Theme.of(context).appBarTheme.backgroundColor,
      width: mediaWidth,
      height: height,
      child: ClipRRect(
        child: BackdropFilter(
          filter: ImageFilter.blur(
            sigmaX: GotoConstants.appBarBlureness,
            sigmaY: GotoConstants.appBarBlureness,
          ),
          child: child,
        ),
      ),
    );
  }
}
