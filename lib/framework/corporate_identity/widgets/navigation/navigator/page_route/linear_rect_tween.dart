import 'dart:ui';

import 'package:flutter/widgets.dart';

/// Linear RectTween with a [Curves.easeOut] curve.
///
/// Less dramatic that the regular [RectTween] used in [Hero] animations.
class LinearRectTween extends RectTween {
  LinearRectTween({
    required Rect begin,
    required Rect end,
  }) : super(begin: begin, end: end);

  @override
  Rect lerp(double t) {
    final easeOutTransformed = Curves.easeOut.transform(t);

    return Rect.fromLTRB(
      lerpDouble(begin!.left, end!.left, easeOutTransformed)!,
      lerpDouble(begin!.top, end!.top, easeOutTransformed)!,
      lerpDouble(begin!.right, end!.right, easeOutTransformed)!,
      lerpDouble(begin!.bottom, end!.bottom, easeOutTransformed)!,
    );
  }
}
