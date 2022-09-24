import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/boxes/goto_static_rounded_box.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/list_tiles/goto_list_space.dart';
import 'package:gotodo/framework/goto_central.dart';

import '../../../framework/corporate_identity/widgets/system/app_bar/goto_sliver_app_bar_template.dart';
import '../../../framework/corporate_identity/widgets/system/boxes/goto_animated_rounded_box.dart';
import '../../../framework/language/goto_dictionary.dart';

class TodosRoot extends StatefulWidget {
  // ignore: prefer_const_constructors_in_immutables
  TodosRoot({Key? key}) : super(key: key);

  @override
  State<TodosRoot> createState() => _TodosRootState();
}

class _TodosRootState extends State<TodosRoot> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final dictionary = GotoDictionary.of(context);

    return GotoSliverAppbarTemplate(
      scrollController: GotoCentral().todosRootScrollController,
      icon: MapEntry(Icons.menu_rounded, () {}),
      title: dictionary.titles.todos,
      body: ListView(
        physics: const BouncingScrollPhysics(),
        children: [
          const GotoListSpace(),
          for (int i = 0; i < 20; i++)
            Padding(
              padding: EdgeInsets.only(
                  bottom: MediaQuery.of(context).size.width * 0.05),
              child: GestureDetector(
                onTap: () {
                  setState(() {
                    _expanded = !_expanded;
                  });
                },
                child: GotoAnimatedRoundedBox(
                  duration: const Duration(milliseconds: 100),
                  height: _expanded
                      ? GotoConstants.listTileHeight * 2
                      : GotoConstants.listTileHeight,
                  child: Container(),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
