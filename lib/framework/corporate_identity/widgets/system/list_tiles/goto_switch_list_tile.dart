import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';

import 'goto_list_tile.dart';

class GotoSwitchListTile extends StatelessWidget {
  const GotoSwitchListTile({
    required this.text,
    required this.onChanged,
    this.value = false,
    Key? key,
  }) : super(key: key);

  final String text;

  final bool value;

  final void Function(bool value) onChanged;

  @override
  Widget build(BuildContext context) {
    return GotoListTile(
      text: text,
      icon: Switch.adaptive(
        value: value,
        onChanged: onChanged,
        activeColor: Theme.of(context).primaryColor,
      ),
    );
  }
}
