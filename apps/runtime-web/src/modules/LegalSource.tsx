const repository = "https://github.com/sergiogallegos/OpenWebHMI";
const exactCommit = /^[0-9a-f]{40}$/.test(__OPENWEBHMI_COMMIT__);
const sourceUrl = exactCommit
  ? `${repository}/tree/${__OPENWEBHMI_COMMIT__}`
  : repository;

/** Visible license and corresponding-source information for this runtime build. */
export function LegalSource() {
  return (
    <footer aria-label="Legal and source information" style={styles.footer}>
      <strong>OpenWebHMI™ {__OPENWEBHMI_VERSION__}</strong>
      <span>AGPL-3.0-only core · MPL-2.0 protocol packages</span>
      <span>Build {__OPENWEBHMI_COMMIT__}</span>
      <a href={sourceUrl} target="_blank" rel="noreferrer">
        Corresponding source
      </a>
      <a href="/legal/LICENSE" target="_blank" rel="noreferrer">
        License text
      </a>
      <a href="/legal/THIRD_PARTY_NOTICES.md" target="_blank" rel="noreferrer">
        Third-party notices
      </a>
    </footer>
  );
}

const styles = {
  footer: {
    display: "flex",
    flexWrap: "wrap" as const,
    gap: 12,
    alignItems: "center",
    marginTop: 24,
    paddingTop: 16,
    borderTop: "1px solid #d9e2ec",
    color: "#52606d",
    fontSize: 12,
  },
};
