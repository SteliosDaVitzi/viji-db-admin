export const privacyConfig = {
  email: 'steliosvitzi@gmail.com',
  updated: '17 October 2026',
  version: 2,
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
    introduction: 'How DBsaur handles your connections, queries, and database data.',
    sections: [
      [
        'Who operates DBsaur',
        'DBsaur is open-source software that you can run locally or self-host. The person or organization running an installation controls its server, connection store, and infrastructure. Project maintainers do not receive your database content through the software. Contact the installation operator for questions about that deployment.',
      ],
      [
        'Database connections and content',
        'DBsaur sends connection details, SQL queries, database metadata, and records between your browser, the DBsaur backend, and the database you select. This is server-assisted processing; database content does not stay exclusively in your browser. Connect only to databases you are authorized to use. The database owner remains responsible for the lawful use of the personal data held in that database.',
      ],
      [
        'Sessions and saved connections',
        'Active connections and credentials are held in backend memory for a session lasting up to eight hours. Disconnecting removes the active session. An HttpOnly, SameSite=Strict cookie named webdb_session identifies the session and is scoped to /api. If server-side saving is configured and you choose to save a connection, its credentials are encrypted in the server connection store until you remove it. Server backups may retain copies under the installation operator’s backup policy. Credentials are never saved in browser storage by DBsaur.',
      ],
      [
        'Browser storage and downloads',
        'Queries that you explicitly save are stored in localStorage under webdb.saved-queries.v1 until you remove them or clear site data. SQL can contain sensitive values: avoid saving secrets or personal data in queries on shared devices. The privacy-notice preference stores only its version and expiry for 180 days under webdb.privacy-notice.v1. Unsaved query drafts and displayed records are held in page memory. JSON exports and clipboard copies remain under your control outside DBsaur. Your browser may cache application assets.',
      ],
      [
        'Purposes and legal grounds',
        'Connection and query data are processed to provide the database operations you request. Installation operators are responsible for identifying and documenting any applicable legal basis for processing personal data. Infrastructure providers may process IP addresses, request times, and technical connection information. Contact the installation operator for its privacy notice, logging, and retention details. This software notice does not certify a deployment as compliant with data-protection law.',
      ],
      [
        'Recipients and international transfers',
        'Requests reach the DBsaur server and your selected database server, together with hosting or network providers needed to deliver them. DBsaur includes no analytics, advertising, tracking pixels, or remote font services. Hosting locations, infrastructure recipients, and any international transfer safeguards depend on the deployment; request those details from its operator before submitting sensitive data.',
      ],
      [
        'Your GDPR rights',
        'Applicable data-protection rights and the process for exercising them depend on your jurisdiction and the installation. Contact the installation operator or the organization responsible for the data; do not send credentials or database records to project maintainers. The operator must provide any legally required privacy disclosures and request procedures. DBsaur includes no automated decision-making or profiling feature.',
      ],
      [
        'Control and contact',
        `Remove saved queries in the query library, remove saved connections on the connection screen, disconnect sessions, and clear your notice preference through Privacy settings in the footer. For data held by your database or a separate deployment, contact its operator. For questions about the software itself, write to ${privacyConfig.email}.`,
      ],
    ],
  },
  terms: {
    title: 'Terms of use',
    introduction: 'The conditions for using DBsaur to access and manage databases.',
    sections: [
      [
        'Operator and scope',
        'DBsaur is a local, single-user database tool with no built-in multi-user authentication or per-user authorization. Keep the backend private. Installation operators are responsible for access controls and any additional terms for their hosted service.',
      ],
      [
        'Open-source license',
        'DBsaur source code is distributed under the MIT license, including its warranty disclaimer and limitation of liability. These usage notes do not restrict the rights granted by that license. Third-party dependencies retain their own licenses. The full license and notices are included in the project repository.',
      ],
      [
        'Authorized use',
        'Use DBsaur only with databases, accounts, and records you are authorized to access. You are responsible for the legality of your queries, the data you process, and any permissions required from database owners or individuals. Do not attempt to bypass security controls, disrupt services, access another user’s information, or use the tool unlawfully.',
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
        'DBsaur is provided as is, without warranty, as set out in the MIT license. Features, compatibility, availability, and displayed results may change or contain errors. Verify important results independently. There is no guaranteed support, maintenance schedule, security certification, or fitness for a particular purpose.',
      ],
      [
        'Responsibility and statutory rights',
        'You remain responsible for the database operations you request. To the extent permitted by law, the operator is not responsible for losses caused by unauthorized use, incorrect instructions, third-party systems, or failure to maintain backups. Nothing in these terms excludes liability that cannot legally be excluded or limits mandatory consumer or data-protection rights.',
      ],
      [
        'Privacy, third parties, and changes',
        'The Privacy policy explains how the software handles data. Third-party database and hosting services are subject to their own terms. Installation operators must provide any required deployment-specific disclosures. These notes may change as the software changes; the date above identifies the current version.',
      ],
    ],
  },
} as const
