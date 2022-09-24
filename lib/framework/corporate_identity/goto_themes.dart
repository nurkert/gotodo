import 'package:flutter/material.dart';

class GoToThemes {
  static const Color goToGreen = Color.fromARGB(255, 0, 204, 0);
  static const Color goToJapaneseLaurel = Color.fromARGB(255, 0, 143, 0);
  static const Color goToSnowyMint = Color.fromARGB(255, 191, 255, 191);
  static const Color goToMintGreen = Color.fromARGB(255, 128, 255, 128);

  static const Color gotoDarkGrey = Color.fromRGBO(33, 33, 33, 1);
  static const Color gotoNotSoDarkGrey = Color.fromARGB(255, 94, 94, 94);
  static const Color gotoLightGrey = Color.fromRGBO(189, 189, 189, 1);
  static const Color gotoLighterGrey = Color.fromRGBO(238, 238, 238, 1);

  static const Color lightFrostedGlass = Color.fromRGBO(255, 255, 255, 0.4);
  static const Color darkFrostedGlass = Color.fromRGBO(0, 0, 0, 0.5);

  static const double appBarTextSize = 17.5;
  static const double appBarIconSize = 20;

  static final darkTheme = ThemeData(
    textSelectionTheme: TextSelectionThemeData(
      cursorColor: Colors.red,
      selectionColor: Colors.green,
      selectionHandleColor: Colors.blue,
    ),
    brightness: Brightness.dark,
    colorScheme: const ColorScheme.dark(),
    primaryColor: goToGreen,
    // splashColor: gotoDarkGrey,
    backgroundColor: Colors.black,
    dialogBackgroundColor: gotoDarkGrey,
    shadowColor: gotoDarkGrey,

    appBarTheme: const AppBarTheme(
      backgroundColor: darkFrostedGlass,
      foregroundColor: Colors.white,
      titleTextStyle: TextStyle(
        fontSize: appBarTextSize,
        fontWeight: FontWeight.w500,
        color: Colors.white,
      ),
      toolbarTextStyle: TextStyle(
        backgroundColor: gotoLightGrey,
      ),
    ),
  );

  static final lightTheme = ThemeData(
    textSelectionTheme: TextSelectionThemeData(
      cursorColor: Colors.red,
      selectionColor: Colors.green,
      selectionHandleColor: Colors.blue,
    ),
    brightness: Brightness.light,
    colorScheme: const ColorScheme.light(),
    primaryColor: goToGreen,
    // splashColor: gotoLighterGrey,
    backgroundColor: Colors.white,
    dialogBackgroundColor: gotoLighterGrey,
    shadowColor: gotoLightGrey,
    appBarTheme: const AppBarTheme(
      backgroundColor: lightFrostedGlass,
      foregroundColor: Colors.black,
      titleTextStyle: TextStyle(
        fontSize: appBarTextSize,
        fontWeight: FontWeight.w500,
        color: Colors.black,
      ),
      toolbarTextStyle: TextStyle(
        backgroundColor: gotoNotSoDarkGrey,
      ),
    ),
  );
}
