import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/input/goto_segmented_control.dart';

import '../../../../framework/corporate_identity/widgets/system/input/goto_text_input_field.dart';
import '../../../../framework/corporate_identity/widgets/system/list_tiles/goto_list_space.dart';
import '../../../../framework/language/goto_dictionary.dart';
import 'priority_settings/priority_options_selectior.dart';

class AddTodoDialog extends StatefulWidget {
  const AddTodoDialog({
    required this.isExtended,
    Key? key,
  }) : super(key: key);

  /// Whether the dialog is extended or not
  final bool isExtended;

  @override
  _AddTodoDialogState createState() => _AddTodoDialogState();
}

class _AddTodoDialogState extends State<AddTodoDialog> {
  @override
  Widget build(BuildContext context) {
    final dictionary = GotoDictionary.of(context);
    return Column(
      mainAxisAlignment: MainAxisAlignment.start,
      children: [
        // GotoListSpace(),
        GotoTextInputField(
          hintText: dictionary.addItem.todo,
          maxLines: 1,
          // autofocus: true,
        ),
        const GotoListSpace(),
        GotoTextInputField(
          hintText: dictionary.addItem.notes,
          maxLines: null,
        ),
        AnimatedSwitcher(
          duration: const Duration(milliseconds: 250),
          child: widget.isExtended
              ? Column(
                  children: [
                    const GotoListSpace(),
                    const PriorityOptionsSelectior(),
                    const GotoListSpace(),
                    GotoSegmentedControl(
                      onValueChanged: (int? value) {},
                      segments: const ["-", "1", "4", "16", "64", "256"],
                    ),
                  ],
                )
              : Container(),
        )
        // GotoListSpace(),
      ],
    );
  }
}
