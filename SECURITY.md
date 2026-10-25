# Security and privacy

FrameLab is a local experimental application. No account, telemetry upload, paid API, arbitrary provider URL or remote command execution is included.

- Tauri capability permits only named app commands for the local main window. Mutations resolve discovered CS2 profile IDs; file imports use native dialogs. No generic shell/filesystem plugin is exposed to web content.
- PresentMon is a fixed official binary checked by size and SHA-256. No game injection, memory writes or security/anti-cheat changes are performed. This does not establish a VAC/FACEIT guarantee.
- GSI listens only on loopback with a random route/token, bounded bodies and timeouts; map name/freshness are retained, player fields are ignored. A trusted local account/host remains an assumption.
- SQLite stores full configuration backups locally, unencrypted. Protect the user account and backups. Runtime databases, replay files and raw captures must never be attached to a public issue.
- Unknown config versions, duplicate keys, out-of-allowlist values, symlinks, conflicting changes, invalid metrics and uncertain recoveries fail closed. A host-level adversary or an external writer racing replacement is outside an absolute recovery guarantee.
- Run the app as a normal user with appropriate tracing permissions, not as a general-purpose elevated UI. FrameLab does not add group memberships, disable Windows protections or install drivers.

Report a vulnerability privately through the repository's security-advisory reporting facility when enabled. Do not post exploit details, player data, machine-identifying paths or credentials in a public issue. State the version, affected command and a synthetic reproduction.

Türkçe: Veriler cihazda kalır; yedekler şifrelenmez. Çalışma veritabanını, replay dosyalarını veya kişisel yollar içeren kayıtları paylaşma. Güvenlik bildirimlerini özel kanaldan ilet. Bilinmeyen ayar sürümü veya çakışma durumunda dosya ezilmez.
