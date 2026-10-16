export const privacyConfig = {
  owner: 'Styliannos Vitzilaios',
  address: 'Sitakis 55, Athens, Greece, 11142',
  email: 'steliosvitzi@gmail.com',
  updated: '16 October 2026',
  version: 1,
  storageKey: 'webdb.privacy-notice.v1',
  retentionDays: 180,
}

export function validPrivacyChoice(raw: string | null, now = Date.now()): boolean {
  try {
    const value = JSON.parse(raw || 'null')
    return (
      value?.version === privacyConfig.version &&
      typeof value.expires === 'number' &&
      value.expires > now &&
      value.expires <= now + privacyConfig.retentionDays * 86400000
    )
  } catch {
    return false
  }
}

export const legalDocuments = {
  privacy: {
    title: 'Privacy policy',
    introduction: 'How WebDB handles your connections, queries, and database data.',
    sections: [
      [
        'Who operates WebDB',
        `WebDB is a Vijitek tool operated by ${privacyConfig.owner}, ${privacyConfig.address}. Contact ${privacyConfig.email} for privacy questions. If you use a separately hosted installation, its operator is responsible for that deployment and its infrastructure.`,
      ],
      [
        'Database connections and content',
        'WebDB sends connection details, SQL queries, database metadata, and records between your browser, the WebDB backend, and the database you select. This is server-assisted processing; database content does not stay exclusively in your browser. Connect only to databases you are authorized to use. The database owner remains responsible for the lawful use of the personal data held in that database.',
      ],
      [
        'Sessions and saved connections',
        'Active connections and credentials are held in backend memory for a session lasting up to eight hours. Disconnecting removes the active session. An HttpOnly, SameSite=Strict cookie named webdb_session identifies the session and is scoped to /api. If server-side saving is configured and you choose to save a connection, its credentials are encrypted in the server connection store until you remove it. Server backups may retain copies under the installation operator’s backup policy. Credentials are never saved in browser storage by WebDB.',
      ],
      [
        'Browser storage and downloads',
        'Queries that you explicitly save are stored in localStorage under webdb.saved-queries.v1 until you remove them or clear site data. SQL can contain sensitive values: avoid saving secrets or personal data in queries on shared devices. The privacy-notice preference stores only its version and expiry for 180 days under webdb.privacy-notice.v1. Unsaved query drafts and displayed records are held in page memory. JSON exports and clipboard copies remain under your control outside WebDB. Your browser may cache application assets.',
      ],
      [
        'Purposes and legal grounds',
        'Connection and query data are processed to provide the database operations you request. Where personal data are involved, the relevant legal basis depends on the relationship and deployment: performance of a requested service or legitimate interests in operating and securing that service. Infrastructure providers may process IP addresses, request times, and technical connection information to deliver and protect the site. Contact the installation operator for its logging and retention details.',
      ],
      [
        'Recipients and international transfers',
        'Requests reach the WebDB server and your selected database server, together with hosting or network providers needed to deliver them. WebDB includes no analytics, advertising, tracking pixels, or remote font services. Hosting locations, infrastructure recipients, and any international transfer safeguards depend on the deployment; request those details from its operator before submitting sensitive data.',
      ],
      [
        'Your GDPR rights',
        'Where applicable, you may request access, correction, erasure, restriction, portability, or object to processing. Where processing relies on consent, you may withdraw it without affecting earlier lawful processing. Contact the operator using the address below; do not send credentials or database records with your request. We respond without undue delay, normally within one month; permitted extensions will be explained. You may complain to the Hellenic Data Protection Authority (dpa.gr) or your local supervisory authority. WebDB does not use your data for automated decisions or profiling.',
      ],
      [
        'Control and contact',
        `Remove saved queries in the query library, remove saved connections on the connection screen, disconnect sessions, and clear your notice preference through Privacy settings in the footer. For data held by your database or a separate deployment, contact its operator. For this Vijitek tool, write to ${privacyConfig.email}.`,
      ],
    ],
  },
  terms: {
    title: 'Terms of use',
    introduction: 'The conditions for using WebDB to access and manage databases.',
    sections: [
      [
        'Operator and scope',
        `WebDB is provided as a Vijitek tool by ${privacyConfig.owner}, ${privacyConfig.address}. Questions about these terms can be sent to ${privacyConfig.email}. These terms apply to use of this tool; a separately hosted installation may have additional terms from its operator.`,
      ],
      [
        'Authorized use',
        'Use WebDB only with databases, accounts, and records you are authorized to access. You are responsible for the legality of your queries, the data you process, and any permissions required from database owners or individuals. Do not attempt to bypass security controls, disrupt services, access another user’s information, or use the tool unlawfully.',
      ],
      [
        'Database changes and backups',
        'Browsing and query execution are intended for reading data. Separate editing and settings actions can insert, update, or delete records and alter database configuration or structure. Review the target database, values, and confirmation before applying a change. Keep appropriate backups and use database accounts with only the permissions you need. Applied changes may be irreversible.',
      ],
      [
        'Credentials and shared devices',
        'Protect your database credentials and use trusted devices and secure connections. Saved connections belong to the installation’s server store and must only be enabled on a trusted deployment with suitable access controls. Saved queries reside in your browser and may be available to others using that browser profile. Disconnect and remove saved information when appropriate.',
      ],
      [
        'Availability and accuracy',
        'WebDB is provided as available. Features, compatibility, availability, and displayed results may change or contain errors. Verify important results independently before relying on them. To the extent permitted by applicable law, no promise is made that the service will be uninterrupted, error-free, or suitable for every purpose.',
      ],
      [
        'Responsibility and statutory rights',
        'You remain responsible for the database operations you request. To the extent permitted by law, the operator is not responsible for losses caused by unauthorized use, incorrect instructions, third-party systems, or failure to maintain backups. Nothing in these terms excludes liability that cannot legally be excluded or limits mandatory consumer or data-protection rights.',
      ],
      [
        'Privacy, third parties, and changes',
        'The Privacy policy explains how the tool handles data. Third-party database and hosting services are subject to their own terms. We may update these terms as the tool changes; the date above identifies the current version. Greek law applies to the extent permitted by law, without removing mandatory protections or rights to bring proceedings available in your country.',
      ],
    ],
  },
} as const
