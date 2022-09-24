import 'dart:ui';

import 'package:flutter/material.dart';

import '../../../goto_themes.dart';

class GotoIconButton extends StatefulWidget {
  const GotoIconButton({
    required this.onTap,
    required this.icon,
    Key? key,
  }) : super(key: key);

  /// The function to be called when the button is tapped
  final void Function() onTap;

  /// The icon to be displayed
  final IconData icon;

  @override
  _GotoIconButtonState createState() => _GotoIconButtonState();
}

class _GotoIconButtonState extends State<GotoIconButton> {
  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: widget.onTap,
      child: Container(
        height: GoToThemes.appBarIconSize * 1.5,
        width: GoToThemes.appBarIconSize * 1.5,
        alignment: Alignment.center,
        child: Icon(
          widget.icon,
          color: Theme.of(context).appBarTheme.foregroundColor,
          size: GoToThemes.appBarIconSize * 1.5,
        ),
      ),
    );
  }
}
