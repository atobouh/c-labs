Machine Room's offline Arrays & Memory pilot now includes the R04 interface corrections and verification.

- Lesson navigation and all learner/header files stay accessible in narrow layouts.
- Keyboard focus, editor cursor/scroll position, and independent disclosure states survive local updates.
- Escape exits code editors; Shift+Tab moves backward normally. Model shortcuts apply inside their own view.
- Saved edits keep older results labelled as previous submissions. Loading, retry, compiler absence, diagnostics, empty review, and empty search states are checked.

The approved light studio and dark editor remain intact, without editor outlines. The small installer reuses existing WebView2; setup needs internet only if it is absent. Learning content and local GCC/Clang execution work offline. The compiler remains an installed prerequisite.

Four lessons, three exercises, two models, three real labs, and one graded project are implemented. Earlier prerequisites and other chapters remain outlines. Optional AddressSanitizer remains unavailable on the tested local MinGW profile. This prerelease does not claim a complete beginner course or a code sandbox.
