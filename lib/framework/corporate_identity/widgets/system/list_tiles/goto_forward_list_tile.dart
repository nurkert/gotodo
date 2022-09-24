import 'package:flutter/material.dart';

import 'package:gotodo/framework/corporate_identity/goto_constants.dart';

import 'goto_list_tile.dart';

class GotoForwardListTile extends StatelessWidget {
  const GotoForwardListTile({
    required this.text,
    required this.routeName,
    Key? key,
  }) : super(key: key);

  final String text;

  final String routeName;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => Navigator.of(context).pushNamed(routeName),
      child: GotoListTile(
        text: text,
        icon: const Icon(
          Icons.arrow_forward_ios_rounded,
          size: GotoConstants.iconSize / 2,
        ),
      ),
    );
  }
}
