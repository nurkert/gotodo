import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';
import 'package:gotodo/framework/corporate_identity/widgets/navigation/navigator/goto_tab.dart';
import 'package:gotodo/views/tabs/notes/notes_root.dart';

class NotesTab extends GotoTab {
  static const String root = "/";

  const NotesTab({
    Key? key,
  }) : super(
          key: key,
          tabTag: GotoConstants.notesTabTag,
          routes: const {
            root: NotesRoot(),
          },
        );
}
