import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';

import '../../../goto_themes.dart';

class GotoStaticAppbarTemplate extends StatelessWidget {
  const GotoStaticAppbarTemplate({
    Key? key,
    required this.title,
    required this.body,
    this.icon,
    this.leading = true,
    this.limitedBody = true,
  }) : super(key: key);

  /// the heading of the view that is displayed in the middle of the appbar
  final String title;

  /// the widget that represents the actual view
  final Widget body;

  /// icon that is displayed to the right of the title in the appbar
  final MapEntry<IconData, void Function()>? icon;

  /// value that decides whether a back icon should be displayed in the appbar
  final bool leading;

  /// Whether only 90 per cent of the display width is available for the body.
  final bool limitedBody;

  @override
  Widget build(BuildContext context) {
    final appBarTheme = Theme.of(context).appBarTheme;
    double mediaWidth = MediaQuery.of(context).size.width;
    return Scaffold(
      extendBodyBehindAppBar: true,
      backgroundColor: Theme.of(context).backgroundColor,
      appBar: PreferredSize(
        preferredSize: const Size.fromHeight(GotoConstants.appBarHeight),
        child: ClipRect(
          child: BackdropFilter(
            filter: ImageFilter.blur(
              sigmaX: GotoConstants.appBarBlureness,
              sigmaY: GotoConstants.appBarBlureness,
            ),
            child: Container(
              color: Theme.of(context).appBarTheme.backgroundColor,
              padding: EdgeInsets.all(mediaWidth * 0.05),
              child: Stack(
                children: [
                  Align(
                    alignment: Alignment.bottomLeft,
                    child: GestureDetector(
                      onTap: () => Navigator.of(context).pop(),
                      child: Icon(
                        Icons.arrow_back_ios_rounded,
                        color: appBarTheme.foregroundColor,
                        size: GoToThemes.appBarIconSize,
                      ),
                    ),
                  ),
                  if (icon != null)
                    Align(
                      alignment: Alignment.bottomRight,
                      child: GestureDetector(
                        onTap: icon!.value,
                        child: Icon(
                          icon!.key,
                          color: appBarTheme.foregroundColor,
                          size: GoToThemes.appBarIconSize,
                        ),
                      ),
                    ),
                  Align(
                    alignment: Alignment.bottomCenter,
                    child: Text(
                      title,
                      style: appBarTheme.titleTextStyle!,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
      body: Scrollbar(
        child: Padding(
          padding: EdgeInsets.symmetric(
            horizontal: mediaWidth * (limitedBody ? 0.05 : 0),
          ),
          child: body,
        ),
      ),
    );
  }
}
