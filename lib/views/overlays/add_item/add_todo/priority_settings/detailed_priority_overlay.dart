import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/boxes/goto_static_rounded_box.dart';

import '../../../../../framework/corporate_identity/goto_constants.dart';

class DetailedPriorityOverlay extends StatelessWidget {
  const DetailedPriorityOverlay({
    required this.child,
    required this.overlayTag,
    Key? key,
  }) : super(key: key);

  final Widget child;

  final String overlayTag;

  @override
  Widget build(BuildContext context) {
    final mediaWidth = MediaQuery.of(context).size.width;
    return Center(
      child: Hero(
        tag: overlayTag,
        createRectTween: GotoConstants.linearRectTween,
        child: SizedBox(
          height: mediaWidth * 0.8,
          width: mediaWidth * 0.8,
          child: GotoRoundedBox(
            color: Theme.of(context).splashColor.withOpacity(1),
            height: mediaWidth * 0.8,
            child: Container(),
          ),
        ),
      ),
    );
  }
}
