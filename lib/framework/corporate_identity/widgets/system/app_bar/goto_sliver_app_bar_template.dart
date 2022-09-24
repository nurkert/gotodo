import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';

import '../../../goto_themes.dart';

class GotoSliverAppbarTemplate extends StatelessWidget {
  const GotoSliverAppbarTemplate({
    required this.title,
    required this.body,
    this.icon,
    this.scrollController,
    this.limitedBody = true,
    Key? key,
  }) : super(key: key);

  /// the heading of the view that is displayed in the appbar
  final String title;

  /// icon that is displayed to the right of the title in the appbar
  final MapEntry<IconData, void Function()>? icon;

  /// the widget that represents the actual view
  final Widget body;

  /// scroll controller of the list located in the body
  final ScrollController? scrollController;

  /// Whether only 90 per cent of the display width is available for the body.
  final bool limitedBody;

  @override
  Widget build(BuildContext context) {
    double mediaWidth = MediaQuery.of(context).size.width;
    final appBarTheme = Theme.of(context).appBarTheme;
    return Container(
      color: Theme.of(context).backgroundColor,
      child: NestedScrollView(
        controller: scrollController,
        headerSliverBuilder: (context, isScrolled) {
          return <Widget>[
            SliverAppBar(
              elevation: 0,
              expandedHeight: 100,
              automaticallyImplyLeading: false,
              collapsedHeight: GotoConstants.appBarHeight,
              backgroundColor: appBarTheme.backgroundColor,
              title: icon != null
                  ? Align(
                      alignment: Alignment.centerRight,
                      child: Icon(
                        icon!.key,
                        color: appBarTheme.foregroundColor,
                        size: GoToThemes.appBarIconSize,
                      ),
                    )
                  : null,
              flexibleSpace: FlexibleSpaceBar(
                titlePadding: EdgeInsets.all(mediaWidth * 0.05),
                centerTitle: false,
                expandedTitleScale: 2,
                title: BackdropFilter(
                  filter: ImageFilter.blur(
                    sigmaX: GotoConstants.appBarBlureness,
                    sigmaY: GotoConstants.appBarBlureness,
                  ),
                  child: Align(
                    alignment: Alignment.bottomLeft,
                    child: Text(
                      title,
                      style: appBarTheme.titleTextStyle!,
                    ),
                  ),
                ),
                // background: Container(),
              ),
              pinned: true,
            )
          ];
        },
        body: Scrollbar(
          child: Padding(
            padding: EdgeInsets.symmetric(
              horizontal: mediaWidth * (limitedBody ? 0.05 : 0),
            ),
            child: body,
          ),
        ),
      ),
    );
  }
}
