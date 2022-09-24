import 'package:flutter/material.dart';
import 'package:gotodo/framework/corporate_identity/widgets/system/boxes/goto_static_rounded_box.dart';

// ignore: must_be_immutable
class GotoSegmentedControl extends StatefulWidget {
  GotoSegmentedControl({
    required this.onValueChanged,
    required this.segments,
    this.selectedSegment = 0,
    Key? key,
  }) : super(key: key);

  final void Function(int?) onValueChanged;

  final List<String> segments;

  int selectedSegment = 0;

  @override
  State<GotoSegmentedControl> createState() => _GotoSegmentedControlState();
}

class _GotoSegmentedControlState extends State<GotoSegmentedControl> {
  @override
  Widget build(BuildContext context) {
    double mediaWidth = MediaQuery.of(context).size.width;

    double alignmentX =
        (widget.selectedSegment / (widget.segments.length - 1)) * 2 - 1;

    double segmentWidth = mediaWidth / widget.segments.length / 1.5;

    return Padding(
      padding: EdgeInsets.symmetric(horizontal: mediaWidth * 0.05),
      child: GotoRoundedBox(
        height: mediaWidth * 0.1,
        child: Stack(
          children: [
            AnimatedAlign(
              duration: const Duration(milliseconds: 200),
              alignment: Alignment(
                alignmentX,
                0,
              ),
              child: GotoRoundedBox(
                color: Theme.of(context).backgroundColor,
                height: mediaWidth * 0.1,
                width: segmentWidth,
                child: Container(),
              ),
            ),
            Align(
              alignment: Alignment.center,
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  for (int i = 0; i < widget.segments.length; i++)
                    GestureDetector(
                      onTap: () => setState(() {
                        widget.selectedSegment = i;
                        widget.onValueChanged(i);
                      }),
                      child: Container(
                        width: segmentWidth,
                        height: mediaWidth * 0.1,
                        color: Colors.transparent,
                        child: Center(
                          child: Text(widget.segments[i]),
                        ),
                      ),
                    )
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
