# Single-file MediaAgent guide

The complete deliverable is `docs/site/index.html`. Open it in a browser or serve
that one file. No other assets, CDN, build step or internet access are required to
render the guide.

The public guide is hosted at <https://maxiplux.github.io/agentmedia/>.
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
- Sixteen workflow/reference sections, including eight practical scenarios, and 62 expandable command help pages.
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
The original 15 offline CLI example checks remain applicable because the commands
and example contents are unchanged. No provider calls were made for this work.

The scenario section covers article narration, a podcast asset package, a data
infographic, a visual/audio lesson, recurring characters, app visuals and sounds,
a vocal jingle, and bilingual narration. Each has input/output descriptions,
commands, authoring files, an agent prompt and review criteria. The four new batch
manifests passed eight offline validation and dry-run checks. Browser checks
confirmed expandable walkthroughs, command copying and mobile layout.
