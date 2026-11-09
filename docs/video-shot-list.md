# Video shot list (W14, H4, H22)

Recorded and narrated by the author in their own voice. The evaluator downloads the video,
samples frames at chapter boundaries and checks for a subtitle track, so:

- upload real subtitles (SRT), not burned-in only; commit the transcript as `docs/video-transcript.md`;
- put the chapter list below in the video description with timestamps;
- show the pinned commit hash on screen at the start and before each live part
  (`git rev-parse HEAD` in a terminal);
- no hidden driver script; if a script runs a step, show it and say so;
- record on testnet 0.4 once it is live; a v0.3 recording is a rehearsal only.

Target length 8–12 minutes.

| # | Chapter | What is on screen | What to say |
|---|---------|-------------------|-------------|
| 1 | Intro (0:00) | README top, `git rev-parse HEAD` | what LP-0018 asks; what this project is; the commit being shown |
| 2 | Architecture (≈1:00) | docs/walkthrough.md "Parts" table | registry on LEZ, files in Logos Storage, SDK/C ABI, CLI, Basecamp app; why one account per region |
| 3 | Discover (≈2:30) | Basecamp app region list (or `osm discover`) | 72 regions, level, hosted status from the registry |
| 4 | Host (≈3:30) | app host flow (or `osm host --region …`) with progress | dated snapshot, MD5 against Geofabrik, upload with retries, the transaction, confirmation read back |
| 5 | Bulk host and import (≈5:00) | select several regions; import a local file with a wrong byte | one batch transaction; the S130 message naming both MD5s |
| 6 | Download on a second machine or profile (≈6:00) | second Basecamp profile or another computer, Geofabrik blocked | fetch by CID, MD5 verified, no Geofabrik needed |
| 7 | Consumer module (≈7:00) | a template module running inside Basecamp | resolve region → CID + metadata through the SDK |
| 8 | macOS (≈8:00) | the app on macOS | same flows on macOS |
| 9 | Verify like an evaluator (≈8:30) | `scripts/verify-onchain.sh` output | ImageID check, per-entry verification, 25/15 count |
| 10 | Honest limits (≈9:30) | docs/limitations.md | what is not done or not proven |

After recording: put the video URL in `docs/SOLUTION_DRAFT.md` (preflight check 4) and the
transcript in `docs/video-transcript.md` (check 19).
