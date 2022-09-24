import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_themes.dart';
import 'package:gotodo/framework/corporate_identity/others/goto_scroll_behavior.dart';
import 'package:gotodo/framework/goto_home.dart';
import 'package:provider/provider.dart';

import 'providers/goto_framework_provider.dart';

class GoToApp extends StatelessWidget {
  const GoToApp({Key? key}) : super(key: key);

  // This widget is the root of the application.
  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider(
      create: (context) => GotoFrameworkProvider(),
      builder: (context, child) {
        final frameworkProvider = Provider.of<GotoFrameworkProvider>(context);

        return MaterialApp(
          debugShowCheckedModeBanner: false,
          title: frameworkProvider.dictionary.general.appDescription,
          themeMode: frameworkProvider.themeMode,
          theme: GoToThemes.lightTheme,
          darkTheme: GoToThemes.darkTheme,
          // builder: (context, child) {
          //   return ScrollConfiguration(
          //     behavior:physics: BouncingScrollPhysics(),,
          //     child: child!,
          //   );
          // },
          scrollBehavior: GotoScrollBehavior(),
          home: const GoToHome(),
        );
      },
    );
  }
}
