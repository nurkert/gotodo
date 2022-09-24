import 'package:flutter/cupertino.dart';
import 'package:gotodo/framework/corporate_identity/widgets/navigation/navigator/goto_navigators.dart';
import 'package:gotodo/framework/corporate_identity/widgets/navigation/navigator/goto_route_builder.dart';

class GotoTabNavigator extends StatelessWidget {
  const GotoTabNavigator({
    required this.tag,
    required this.routes,
    this.initialRoute,
    Key? key,
  }) : super(key: key);

  /// The tag of the tab
  final String tag;

  final String? initialRoute;

  /// The map of routes to be used by the navigator
  final Map<String, Widget> routes;

  @override
  Widget build(BuildContext context) {
    GlobalKey<NavigatorState> key = GoToNavigators().initializeKey(tag);
    final routeBuilder = GotoRouteBuilder(routes);
    final routeStructure = routeBuilder.build(context);

    return Navigator(
      initialRoute: initialRoute,
      key: key,
      onGenerateRoute: (routeSettings) {
        return CupertinoPageRoute(
          builder: (context) => routeStructure[routeSettings.name]!(context),
          settings: routeSettings,
        );
      },
      observers: [
        HeroController(),
      ],
    );
  }
}
