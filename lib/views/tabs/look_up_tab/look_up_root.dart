import 'package:flutter/material.dart';
import 'package:gotodo/framework/language/goto_dictionary.dart';

class LookUpRoot extends StatelessWidget {
  const LookUpRoot({Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Icon(
          Icons.lock_outline_rounded,
          color: Theme.of(context).primaryColor,
          size: 60,
        ),
        Text(
          GotoDictionary.of(context).general.toBeContinued,
          style: TextStyle(
            color: Theme.of(context).primaryColor,
          ),
        )
      ],
    );
  }
}
