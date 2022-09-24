import 'package:flutter/material.dart';

class GotoListSpace extends StatelessWidget {
  const GotoListSpace({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    double fivePercent = MediaQuery.of(context).size.width * 0.05;
    return SizedBox(
      height: fivePercent,
      width: fivePercent,
    );
  }
}
