# Setup and recovery

## Desktop

Use Windows 11 x64. The per-user NSIS installer and portable ZIP contain FrameLab, PresentMon 2.6.0 and license notices. WebView2 must already be installed. Neither package modifies Windows security settings or installs CS2. Keep the portable executable and its `presentmon` subdirectory together.

Normal user privileges are sufficient for discovery and read-only analysis. PresentMon ETW capture requires an account with the relevant Windows tracing permissions. The upstream [permissions discussion](https://github.com/GameTechDev/PresentMon/blob/v2.6.0/README.md) describes administrator/Performance Log Users requirements. Have the device administrator configure appropriate permissions; FrameLab does not add group membership or request a broad elevated background service. A failed capture causes the experiment to stop and recover.

### CS2 adapter

Steam roots come from the current user's Steam registry location and the normal installation location. `libraryfolders.vdf` discovers additional libraries; `appmanifest_730.acf` scopes the game. The UI explicitly selects a userdata profile when there is more than one.

Only `video.cfg` Version 16 is recognized for writing. The allowlist is MSAA samples and the supported ambient-occlusion/particle/texture detail fields. The current candidate engine uses MSAA and particle/ambient-occlusion candidates. Unknown versions, duplicate keys, unexpected values, links/reparse points and missing files are refused. New CS2 versions require fixture/provenance review and actual compatibility evidence before expanding support.

A replay must be a local `.dem` with the CS2 `PBDEMS2` header. The launcher refuses paths containing console metacharacters. FrameLab does not download replays or workshop content. Start with Steam already running, close existing CS2 sessions and use a replay compatible with your game build. A delegated/exited process stops the experiment rather than attaching to a different game.

### Experiment duration

Up to two candidates, five paired A/B comparisons each and a separate five-pair confirmation can require 20–30 game launches. Each run waits for map telemetry, warms up for 15 seconds, then captures 15–120 seconds. The UI displays a bounded estimate before starting. Scene telemetry is a temporary, token-authenticated loopback GSI configuration; no player fields are requested or saved.

### Recovery

Changes are journaled in `%LOCALAPPDATA%/studio.nexora.framelab/` as resolved by Tauri. Stop waits for owned processes to exit and restores temporary fields. After a force termination, close CS2, reopen FrameLab and scan again. Prepared/applied changes are recovered; deliberately committed changes remain until you choose Restore.

Conflicting user/Steam Cloud changes are not overwritten. A modified temporary GSI file is not deleted. Keep backups and inspect the reported conflict before resolving it. Source backups are local and unencrypted. Do not share the runtime database or replay files in an issue.

## Development

Install official Tauri Windows prerequisites, Rust 1.99.1 (with rustfmt/clippy), Node 24.x and Python 3 for packaging. Run `npm ci --ignore-scripts`, `npm run prepare:tools`, then `npm run desktop`. Build with `npm run desktop:build`.

The tool-preparation script downloads only the pinned official PresentMon executable, verifies its expected size/SHA-256 and retains its upstream license. No runtime update or arbitrary script download is performed by the app.

## Türkçe

Windows 11 x64 kurulum paketini veya taşınabilir ZIP'i kullan. Taşınabilir paketteki `presentmon` klasörünü EXE'den ayırma. WebView2 önceden kurulu olmalı. CS2 ve sürücüler otomatik indirilmez.

Önce sistemi tara. Sadece ölçüm okumak için CSV içe aktar; ayar değişmez. Deney için CS2'yi kapat, bir yerel replay seç ve kullanılacak ayar profilini belirt. Desteklenen ayar sürümü şu an Version 16 ile sınırlıdır. Başka sürümde uygulama yazmayı reddeder; bu bir hata gizleme davranışı değildir.

Ölçüm izni hatasında Windows yöneticisinin PresentMon'un resmî ETW izin rehberini incelemesi gerekir. Uygulama güvenlik ayarı kapatmaz veya kendine yönetici yetkisi vermez. İşlem yarıda kalırsa CS2'yi kapatıp yeniden tara. Çakışan kullanıcı değişiklikleri veya değiştirilmiş geçici telemetri dosyası otomatik ezilmez/silinmez.

Canlı CS2 testi bu sürümün doğrulamasına dahil edilmedi. Arayüzdeki örnek sayılar sentetiktir ve gerçek FPS artışı anlamına gelmez.
