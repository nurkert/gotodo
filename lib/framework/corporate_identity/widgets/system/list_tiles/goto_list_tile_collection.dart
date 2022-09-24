import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';

import '../boxes/goto_static_rounded_box.dart';

class GotoListTileCollection extends StatefulWidget {
  const GotoListTileCollection({
    required this.listTiles,
    this.header,
    this.footer,
    Key? key,
  }) : super(key: key);

  final List<Widget> listTiles;
  final String? header;
  final String? footer;

  @override
  _GotoListTileCollectionState createState() => _GotoListTileCollectionState();
}

class _GotoListTileCollectionState extends State<GotoListTileCollection> {
  @override
  Widget build(BuildContext context) {
    double mediaWidth = MediaQuery.of(context).size.width;
    List<Widget> children = [];
    for (int i = 0; i < widget.listTiles.length; i++) {
      if (i > 0) children.add(const Divider(height: 0));
      children.add(widget.listTiles[i]);
    }
    return Column(
      children: [
        if (widget.header != null)
          Align(
            alignment: Alignment.bottomLeft,
            child: Padding(
              padding: EdgeInsets.only(
                left: mediaWidth * 0.03,
                bottom: mediaWidth * 0.01,
              ),
              child: Text(
                widget.header!,
                style: TextStyle(
                  color: Theme.of(context)
                      .appBarTheme
                      .toolbarTextStyle!
                      .backgroundColor!,
                  fontSize: 12,
                ),
              ),
            ),
          ),
        GotoRoundedBox(
          height: widget.listTiles.length * GotoConstants.listTileHeight,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: children,
          ),
        ),
        if (widget.footer != null)
          Align(
            alignment: Alignment.topLeft,
            child: Padding(
              padding: EdgeInsets.only(
                left: mediaWidth * 0.03,
                top: mediaWidth * 0.01,
              ),
              child: Text(
                widget.footer!,
                style: TextStyle(
                  color: Theme.of(context)
                      .appBarTheme
                      .toolbarTextStyle!
                      .backgroundColor!,
                  fontSize: 12,
                ),
              ),
            ),
          ),
      ],
    );
  }
}
