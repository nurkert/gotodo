import 'dart:developer';

import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/goto_constants.dart';
import 'package:gotodo/views/overlays/add_item/add_todo/priority_settings/detailed_priority_overlay.dart';

import '../../../../../framework/corporate_identity/widgets/navigation/navigator/page_route/overlay_dialog_route.dart';

class GotoWidgetSelector extends StatefulWidget {
  const GotoWidgetSelector({
    required this.selectable,
    required this.onChanged,
    required this.color,
    Key? key,
  }) : super(key: key);

  final Map<String, Widget> selectable;

  final void Function(int)? onChanged;

  final Color color;

  @override
  State<GotoWidgetSelector> createState() => _GotoWidgetSelectorState();
}

class _GotoWidgetSelectorState extends State<GotoWidgetSelector> {
  int _selecetd = 0;

  late PageController pageController;

  @override
  void didChangeDependencies() {
    pageController = PageController(
      viewportFraction: 0.4725,
      initialPage: _selecetd,
    );
    super.didChangeDependencies();
  }

  @override
  Widget build(BuildContext context) {
    double mediaWidth = MediaQuery.of(context).size.width;
    return Column(
      children: [
        SizedBox(
          height: 0.375 * mediaWidth,
          child: PageView(
            controller: pageController,
            onPageChanged: (index) => setState(() {
              _selecetd = index;
            }),
            padEnds: false,
            children: [
              for (int i = 0; i < widget.selectable.length; i++)
                Padding(
                  padding: EdgeInsets.only(
                    left: mediaWidth * 0.05,
                  ),
                  child: GestureDetector(
                    onTap: () {
                      // if unselected widget is clicked, animate to it and select it
                      if (_selecetd != i) {
                        pageController.animateToPage(
                          i,
                          duration: const Duration(milliseconds: 300),
                          curve: Curves.easeInOut,
                        );
                        _selecetd = i;
                      }
                      Navigator.of(context).push(
                        OverlayDialogRoute(
                          builder: (context) {
                            return DetailedPriorityOverlay(
                              child: Container(),
                              overlayTag: widget.selectable.keys.elementAt(i),
                            );
                          },
                          // settings: ModalRoute.of(context)!.settings,
                          settings: const RouteSettings(),
                        ),
                      );
                    },
                    child: Hero(
                      tag: widget.selectable.keys.elementAt(i),
                      child: SizedBox(
                        width: 0.375 * mediaWidth,
                        child: Container(
                          decoration: BoxDecoration(
                            color: widget.color,
                            borderRadius: BorderRadius.only(
                              topLeft: Radius.circular(
                                0 == i ? GotoConstants.circularBorderRadius : 0,
                              ),
                              topRight: Radius.circular(
                                widget.selectable.length - 1 == i
                                    ? GotoConstants.circularBorderRadius
                                    : 0,
                              ),
                            ),
                          ),
                          child: widget.selectable.values.toList().elementAt(i),
                        ),
                      ),
                    ),
                  ),
                ),
              SizedBox(
                width: mediaWidth * 0.15,
              )
            ],
          ),
        ),
        AnimatedPadding(
          duration: const Duration(milliseconds: 250),
          padding: EdgeInsets.only(
            left: _selecetd == widget.selectable.length - 1
                ? mediaWidth * 0.0995
                : mediaWidth * 0.05,
          ),
          child: Align(
            alignment: Alignment.topLeft,
            child: Container(
              // height: 0.075 * mediaWidth,
              height: 0.05 * mediaWidth,
              width: 0.375 * mediaWidth,
              decoration: BoxDecoration(
                color: widget.color,
                borderRadius: const BorderRadius.only(
                  bottomLeft: Radius.circular(
                    GotoConstants.circularBorderRadius,
                  ),
                  bottomRight: Radius.circular(
                    GotoConstants.circularBorderRadius,
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}
