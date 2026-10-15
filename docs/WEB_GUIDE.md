# Single-file FrameChord guide

FrameChord is the project name from 0.6.1 onward. Earlier evidence below keeps its original MediaAgent commands, release names and artifact paths. Existing `mediaagent/` projects remain supported in place. See [rename and upgrade guidance](../assets/skill/references/renaming.md).

The complete deliverable is `docs/site/index.html`. Open it in a browser or serve
that one file. No other assets, CDN, build step or internet access are required to
render the guide.

The FrameChord guide is published at <https://maxiplux.github.io/framechord/>.
The owned custom domain is `framechord.com`; see [domain setup](DOMAIN.md).
The `Publish documentation` GitHub Actions workflow publishes only `docs/site/`
to GitHub Pages after changes to that folder or the workflow on `master`. It can
also be started manually:

```sh
gh workflow run pages.yml --ref master
```

Edit `docs/site/index.html` and push to `master` to update the live guide. Check
the workflow's deployment result before treating an update as published. There is
no build step, and the hosted document is the same standalone file used locally.

The HTML contains:

- Compiled Tailwind CSS 4.3.3 with the guide's component styles in a `<style>` tag.
- Embedded JavaScript for search, navigation and copy buttons.
- Sixteen workflow/reference sections, including eight practical scenarios, and 65 expandable command help pages.
- Four download links carrying the same embedded ZIP of 35 example files.

Example downloads extract into `examples/` in your target project. They retain
relative YAML dependencies. The cast example intentionally requires your own
accepted character references. Credential variable names contain no secret values.

From the repository root, an optional local preview is:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 --directory docs/site
```

Open http://127.0.0.1:8765/ while that process runs. A server is optional for normal
browser use; local-file clipboard restrictions use a selection/copy fallback.

The Tailwind CLI compiled the component rules using `@apply` and utility classes;
there is no runtime Tailwind script. Official build reference:
https://tailwindcss.com/docs/installation/tailwind-cli

Verified after bundling: no external scripts, styles, images or file links;
unique element IDs and valid internal anchors; embedded ZIP integrity and all 35
files; browser command search and copying; a mobile layout without page overflow.
The 0.3.0 guide and extracted examples pass 22 offline initialization, dry-run and
validation checks. No provider calls were made for this work. Earlier mobile
layout checks apply to the base layout; the updated setup and command sections
were rechecked in a desktop browser.

The scenario section covers article narration, a podcast asset package, a data
infographic, a visual/audio lesson, recurring characters, app visuals and sounds,
a vocal jingle, and bilingual narration. Each has input/output descriptions,
commands, authoring files, an agent prompt and review criteria. The four new batch
manifests passed eight offline validation and dry-run checks. Browser checks
confirmed expandable walkthroughs, command copying and mobile layout.

Version 0.3.0 updates initialization to a resumable keyboard-driven checklist, optional masked credential entry, OpenAI/Fish defaults, local readiness and a reviewed current-stable model policy. Its 63 command pages and 35 downloadable examples match the updated executable and templates. New-model execution uses mocked requests; this documentation update makes no paid provider requests.

Version 0.3.1 documents the executable/configuration folder name conflict, early destination selection, full-path display and `--project` precedence. The command reference is regenerated from the corrected CLI. The example archive remains unchanged.

Version 0.4.0 adds public installer links, the install/update/rollback workflow and two generated command pages. The guide remains a standalone HTML document; the installer scripts are separate downloads used only when explicitly installing software.

The 0.4.0 public guide and both installer downloads were fetched from GitHub Pages and matched byte-for-byte with the merged source on October 15, 2026. Public native installation, self-update and offline rollback passed on all four supported release targets; see [the recorded results](STATUS.md#user-installation-and-updates-040).

Version 0.5.1 updates the setup walkthrough and generated command help for Google-first defaults, the explicit OpenAI preset, built-in speech voices, target-project dotenv creation/update and default installation of Codex/Claude project skills. Examples that name other providers remain explicit provider choices and require their corresponding credentials.


Version 0.6.1 renames the guide, all 65 generated command pages, installers and
all four embedded example archives to FrameChord. The upgrade note distinguishes
new `framechord/` projects from preserved legacy `mediaagent/` directories.
The custom domain's registration is separate from DNS and HTTPS deployment.
