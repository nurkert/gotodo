import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';
import 'package:gotodo/views/overlays/add_item/add_item_overlay.dart';

import '../../navigation/navigator/page_route/overlay_dialog_route.dart';

class GoToFloatingActionButton extends StatefulWidget {
  const GoToFloatingActionButton({Key? key}) : super(key: key);

  @override
  State<GoToFloatingActionButton> createState() =>
      _GoToFloatingActionButtonState();
}

class _GoToFloatingActionButtonState extends State<GoToFloatingActionButton>
    with SingleTickerProviderStateMixin {
  late double _scale;
  late AnimationController _controller;

  @override
  void initState() {
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 50),
      lowerBound: 0.0,
      upperBound: 0.2,
    );
    _controller.addListener(() {
      setState(() {});
    });
    super.initState();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    _scale = 1 - _controller.value;
    return Transform.scale(
      scale: _scale,
      child: SizedBox(
        height: GotoConstants.actionButtonSize,
        width: GotoConstants.actionButtonSize,
        child: GestureDetector(
          onTap: () async {
            // await _controller.forward();
            // await _controller.reverse();
            HapticFeedback.mediumImpact();
            Navigator.of(context).push(
              OverlayDialogRoute(
                builder: (context) {
                  return const AddItemOverlay();
                },
                // settings: ModalRoute.of(context)!.settings,
                settings: const RouteSettings(),
              ),
            );
          },
          child: Hero(
            tag: GotoConstants.addItemHeroTag,
            createRectTween: GotoConstants.linearRectTween,
            child: Material(
              color: Theme.of(context).primaryColor,
              elevation: 2,
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(32),
              ),
              child: Icon(
                Icons.add_rounded,
                size: GotoConstants.iconSize,
                color: Theme.of(context).backgroundColor,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
