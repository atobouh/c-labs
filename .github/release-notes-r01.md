Machine Room R01 is a Windows desktop host for the C-Lab course catalog.

Download **Machine.Room_0.1.0_x64-setup-slim.exe** for normal Windows installation. It uses an existing WebView2 runtime; setup downloads the runtime only if it is missing. The app includes the approved local studio UI, the three course outlines, verified Arrays & Memory pilot records, and the C lexicon. Course search and record lookup use the local SQLite catalog. Open-lab drafts use a separate local database and are preserved by transactional migrations.

Lesson, exercise, animation, and lab interactions, project grading, and chapter progression are still previews. This release does not compile or run learner code.

The application works offline after installation. The original 210 MiB installer remains available for a fresh computer that lacks WebView2 and must install without internet. The installers are unsigned.
