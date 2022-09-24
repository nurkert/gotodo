import 'dart:math';
import 'dart:ui';

import 'package:flutter/material.dart';

import '../../../goto_constants.dart';

class GotoTextInputField extends StatelessWidget {
  const GotoTextInputField({
    this.controller,
    this.hintText,
    this.maxLines = 1,
    this.autofocus = false,
    Key? key,
  }) : super(key: key);

  /// The controller for the text field
  final TextEditingController? controller;

  /// The hint text to be displayed
  final String? hintText;

  /// The maximum number of lines to show
  final int? maxLines;

  /// if true, the text input field will be focused when the widget is built
  final bool autofocus;

  @override
  Widget build(BuildContext context) {
    double mediaWidth = MediaQuery.of(context).size.width;

    // double height = mediaWidth * 0.1 * maxLines;

    Color fieldColor = Theme.of(context).splashColor;

    InputBorder border = OutlineInputBorder(
      borderSide: BorderSide(width: 0, color: fieldColor),
      borderRadius: BorderRadius.circular(
        GotoConstants.circularBorderRadius,
      ),
    );

    return Padding(
      padding: EdgeInsets.symmetric(horizontal: mediaWidth * 0.05),
      child: TextField(
        style: TextStyle(
          fontSize: 15,
          color: Theme.of(context).appBarTheme.titleTextStyle!.color,
        ),

        cursorColor: Theme.of(context).cardColor,
        // selectionHeightStyle: BoxHeightStyle(),
        autofocus: autofocus,
        maxLines: maxLines,

        decoration: InputDecoration(
          fillColor: fieldColor,
          hintText: hintText,
          filled: true,
          // isDense: true,
          isCollapsed: true,
          contentPadding: EdgeInsets.symmetric(
            horizontal: mediaWidth * 0.03,
            vertical: mediaWidth * 0.03,
          ),
          border: border,
          focusedBorder: border,
          enabledBorder: border,

          // borderRadius: const BorderRadius.all(
          //   Radius.circular(GotoConstants.circularBorderRadius),
          // ),
        ),
        controller: controller,
      ),
    );
  }
}
