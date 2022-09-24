import 'package:flutter/cupertino.dart';
import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/app_bar/goto_sliver_app_bar_template.dart';

import '../../../framework/language/goto_dictionary.dart';

class NotesRoot extends StatefulWidget {
  const NotesRoot({Key? key}) : super(key: key);

  @override
  State<NotesRoot> createState() => _NotesRootState();
}

class _NotesRootState extends State<NotesRoot> {
  @override
  Widget build(BuildContext context) {
    final dictionary = GotoDictionary.of(context);

    return GotoSliverAppbarTemplate(
      title: dictionary.titles.notes,
      body: ListView(
        physics: const BouncingScrollPhysics(),
        children: [],
      ),
    );
  }
}
