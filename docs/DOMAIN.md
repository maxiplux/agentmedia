# Connecting framechord.com

The user selected FrameChord and owns `framechord.com`. The custom domain's DNS
and HTTPS deployment must be verified independently of repository publication.
Until then, use `https://maxiplux.github.io/framechord/` for the guide and installers.
Do not label the custom domain live until it serves the intended guide over HTTPS.

At the domain's DNS provider, configure these records for GitHub Pages:

| Type | Host | Value |
| --- | --- | --- |
| A | @ | 185.199.108.153 |
| A | @ | 185.199.109.153 |
| A | @ | 185.199.110.153 |
| A | @ | 185.199.111.153 |
| CNAME | www | maxiplux.github.io |

Use the provider's default TTL. Review conflicting web-hosting or parking records
for these same hosts before replacing them. Preserve unrelated email, verification
and other DNS records. The `www` value has no repository path.

In the `maxiplux/framechord` repository's Settings → Pages, select
`framechord.com` as the custom domain, let GitHub check DNS and issue its
certificate, then enable Enforce HTTPS when available. Verify the apex domain,
`www`, the guide anchors and both installer downloads. GitHub Actions deployment
uses the Pages setting; a checked-in `CNAME` file is not required by this workflow.

The GitHub repository and DNS are separate services. Renaming the repository
does not configure the registrar's DNS records. GitHub account domain ownership
verification can be added separately through the account's Pages settings.

Values reviewed October 15, 2026 against the official
[GitHub Pages custom-domain documentation](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site).
