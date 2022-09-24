import 'package:flutter/material.dart';

import 'package:gotodo/framework/corporate_identity/goto_constants.dart';

class GotoListTile extends StatelessWidget {
  const GotoListTile({
    required this.text,
    this.icon,
    Key? key,
  }) : super(key: key);

  final String text;
  final Widget? icon;

  @override
  Widget build(BuildContext context) {
    double mediaWidth = MediaQuery.of(context).size.width;
    return Container(
      color: Colors.transparent,
      child: Padding(
        padding: EdgeInsets.symmetric(horizontal: mediaWidth * 0.03),
        child: SizedBox(
          height: GotoConstants.listTileHeight,
          child: Stack(
            children: [
              Align(
                alignment: Alignment.centerLeft,
                child: Text(text),
              ),
              if (icon != null)
                Align(
                  alignment: Alignment.centerRight,
                  child: icon,
                ),
            ],
          ),
        ),
      ),
    );
  }
}
