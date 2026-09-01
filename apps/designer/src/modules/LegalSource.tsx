const repository = "https://github.com/sergiogallegos/OpenWebHMI";
const exactCommit = /^[0-9a-f]{40}$/.test(__OPENWEBHMI_COMMIT__);
const sourceUrl = exactCommit
  ? `${repository}/tree/${__OPENWEBHMI_COMMIT__}`
  : repository;

/** Visible license and corresponding-source information for this Designer build. */
export function LegalSource() {
  return (
    <footer aria-label="Legal and source information" style={styles.footer}>
      <strong>OpenWebHMI™ {__OPENWEBHMI_VERSION__}</strong>
      <span>AGPL-3.0-only core · MPL-2.0 protocol packages</span>
      <span>Build {__OPENWEBHMI_COMMIT__}</span>
      <a href={sourceUrl} target="_blank" rel="noreferrer">
        Corresponding source
      </a>
      <a href={`${repository}/blob/${exactCommit ? __OPENWEBHMI_COMMIT__ : "main"}/LICENSE-POLICY.md`} target="_blank" rel="noreferrer">
        License map
      </a>
      <span>Canonical texts and third-party notices are bundled with the app.</span>
    </footer>
  );
}

const styles = {
  footer: {
    display: "flex",
    flexWrap: "wrap" as const,
    gap: 10,
    alignItems: "center",
    padding: "8px 12px",
    borderTop: "1px solid #d9e2ec",
    background: "#ffffff",
    color: "#52606d",
    fontSize: 11,
  },
};
