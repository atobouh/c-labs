Machine Room R01 is a Windows desktop host for the C-Lab course catalog.

The installer includes the approved local studio UI, the three course outlines, verified Arrays & Memory pilot records, the C lexicon, and the WebView2 offline installer. Course search and record lookup use the local SQLite catalog. Open-lab drafts use a separate local database and are preserved by transactional migrations.

Lesson, exercise, animation, and lab interactions, project grading, and chapter progression are still previews. This release does not compile or run learner code.

The installer is unsigned, so Windows SmartScreen may show a reputation warning. The application itself makes no runtime network requests; this first installer is larger because it carries WebView2 for offline installation.
