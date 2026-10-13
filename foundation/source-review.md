# F01 source review

Reviewed 2026-10-13 for source identity, useful scope, citation locator, and redistribution decision. “Reviewed” means these pages and evidence were inspected; it is not a blanket permission to reproduce content or legal advice. Research-source content is never required after install. The pilot uses original explanations and exercises.

## C semantics and tool references

**WG14 C17 draft N2176** is the primary semantics reference. The official [WG14 document log](https://www.open-std.org/jtc1/sc22/wg14/www/wg14_document_log_v2.htm) identifies N2176 as Gustedt’s proposed C17 document dated 2017-10-09. Cite the official [N2176 PDF](https://www.open-std.org/jtc1/sc22/wg14/www/docs/n2176.pdf) by clauses: §6.2.5 ¶20 (array type/contiguous objects); §6.5.2.1 ¶2 (subscript definition); §6.3.2.1 ¶3 (array conversion and exceptions); §6.5.6 ¶¶8–9 (pointer range, one-past and dereference); §6.7.6.3 ¶7 (array parameter adjustment).

Access limitation: the official endpoint returned HTTP 200 and a 533-page PDF, but its text-extraction permissions prevented local/PDF-parser text review. The section text and locator were cross-checked against a 534-page [full-text copy titled ISO/IEC 9899:2017 C17 ballot N2176](https://bolha.dev/blau_araujo/cblc/raw/commit/2bbee7c9ff6fcc10018015f100f5f27e24c0508b/c17/iso-iec-9899-2017.pdf), a secondary mirror. It was used only to verify the named clause text; it is not our citation URL, it is not included in the project, and no prose was copied. The official original remains the learner citation. This limitation must stay visible if N2176 cannot be opened in a learner's browser. Do not make the conceptual diagram look like actual machine addresses.

**GCC 17.0.0 online manual** is the official compiler-option reference; the version is stated on the current manual landing page. The warning-option section is §3.9. It confirms `-Wall`, `-Wextra`, and `-Wpedantic`; warnings do not become errors unless `-Werror` is supplied. The manual is distributed under GFDL 1.3 or later with specified invariant/cover-text obligations. Use linked documentation only; do not bundle its text or images. Pin and report the actual installed compiler in each lab.

**LLVM AddressSanitizer manual** supports later tool-behavior explanations: its Introduction and Limitations sections describe the detected bug classes and limits, and Supported Platforms lists its platform scope. The page states no fixed docs release. No page-specific documentation reuse grant was identified, so link to it and do not reproduce its text or examples. An ASan pass must never be described as proof of correctness.

## Course-design research and reuse

| Source | Verified scope/version | Reuse decision |
| --- | --- | --- |
| [CS50x Week 1: C](https://cs50.harvard.edu/x/weeks/1/) | Course page title CS50x 2026; C, compilation, types, control flow and problem set. Some embedded assets point into a 2025/fall directory, so content revision needs rechecking before a later comparison. | Official [2026 course license](https://cs50.harvard.edu/x/2026/license/) is CC BY-NC-SA 4.0, subject to exclusions for third-party items. Link only. Do not adapt course prose/problems into this app. |
| [MIT 6.S096 Introduction to C and C++](https://ocw.mit.edu/courses/6-s096-introduction-to-c-and-c-january-iap-2013/) | January IAP 2013; mixed C/C++; daily assignments and a small individual project. Historical reference, not a C17 sequence. | MIT [terms](https://ocw.mit.edu/pages/privacy-and-terms-of-use/) set CC BY-NC-SA 4.0 by default when a material has no different notice, and exclude some third-party material. Noncommercial, attribution, ShareAlike, and item-specific terms matter. Link only. |
| [Beej’s Guide to C](https://beej.us/guide/bgc/) | Official current HTML route; the foreword has no numbered revision, states copyright ©2021, and assumes prior programming knowledge. | [Author’s section 1.8](https://beej.us/guide/bgc/html/split/foreword.html#copyright-and-distribution) applies CC BY-NC-ND 3.0 to prose (with a narrow translation exception) and separately puts its C sample code in the public domain. Link only; no prose or code copied. |
| [Modern C](https://gustedt.gitlabpages.inria.fr/modern-c/) | Author identifies the public C23 text as latest on 2024-10-15; the third-edition print/eBook appeared 2025-09-01. Page says contents match, though numbering/layout can differ. | [Author’s terms, §§5.1–5.2](https://gustedt.gitlabpages.inria.fr/modern-c/) give prose CC BY-NC-ND 4.0 and code MIT with notice conditions. Link only for the pilot; any later code reuse requires per-file attribution and notice. |

These books and courses help compare progression and practice formats. They are not the pilot’s content source. License terms permit some uses, but the offline application’s distribution/commercial status is not decided here; linking and original authorship avoid relying on those permissions.

## F02 supplement: integer-range contract

The sensor project uses `int` for the sum because its maximum accepted total is 30,000. N2176 §5.2.4.2.1 paragraph 1 requires `INT_MAX` to be at least 32,767. The official PDF remains the learner citation; its text-extraction restriction is unchanged. This additional locator was cross-checked in a full-text N2176 copy hosted by the University of Western Australia, used only as a secondary verification aid: <https://teaching.csse.uwa.edu.au/units/CITS2002/resources/n2176.pdf>.

## Status and follow-up

All seven source records now have actual retrieval dates, the best version detail shown by the source, a citation/relevance note, and a conservative reuse decision. `sources.json` is the machine-readable record; this file is the human audit. The source pages can change, so re-check applicable versions, notices, and individual asset rights before later publishing or bundling anything. F02 should settle the pilot’s exact claim-to-clause citations and API contract; F03 must cite the verified C17 clauses without copying standard prose.
