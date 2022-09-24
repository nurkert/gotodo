import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'framework/goto_app.dart';

void main() {
  // to execute code before the widget tree
  WidgetsFlutterBinding.ensureInitialized();

  // to prevent landscape mode
  SystemChrome.setPreferredOrientations([DeviceOrientation.portraitUp]);

  runApp(const GoToApp());
}
