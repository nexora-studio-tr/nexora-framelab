# Capture comparison / Kayıt karşılaştırması

## English

In **Results → Compare two captures**, choose the baseline CSV, then the candidate CSV. Cancelling either native file picker leaves the report history unchanged. This works without installing or running CS2, collecting new telemetry or changing game settings. The browser offers a clearly synthetic example instead of accessing your files.

The native app stores the report locally; **Export report** writes JSON to a new file you choose. It never overwrites an existing export. The command-line equivalent is:

```sh
framelab-cli compare baseline.csv candidate.csv
```

The CLI emits a comparison JSON to stdout, with no application database or settings changes. Either CSV can be up to 64 MiB with at most one million rows. Existing single-process and dominant-swapchain parsing rules still apply. Duplicate case-insensitive column names are rejected.

### What the checks mean

Percentage changes are withheld unless all five checks pass:

| Check | Rule |
| --- | --- |
| Frame metric | Both parsers selected the same column type: `FrameTime`, `CPUFrameTime` or `MsBetweenPresents`. Different types are conservatively kept separate. |
| Distinct captures | SHA-256 fingerprints differ after removing a UTF-8 BOM. This detects identical imports, not independent measurements. |
| Frame count | Each selected stream contains at least 1,000 valid frames. This is a quality floor, not a statistical confidence guarantee. |
| Duration | The selected frame-duration sums differ by at most 10% of the longer capture. Each already meets the parser's 10-second minimum. |
| Quality | Neither contains a detected major loading stall, collector-loss warning or excluded secondary stream. This cannot detect every collector loss or bad scene. |

Valid captures still show their absolute metrics when a comparison check fails. The panel identifies each failed check and withholds percentage deltas. Missing optional GPU timing does not block presented-frame comparison. Displayed-FPS deltas require positive displayed timing for every selected frame in both files; otherwise they remain unavailable.

For each metric, percentage change is `(candidate / baseline - 1) × 100`. Higher FPS and 1% low are better; lower p95/p99 frame time is better. Stutters are normalized per minute, and their delta is an absolute rate difference, avoiding raw-count comparisons across different durations. The trace axis is normalized to each capture's length; it is not a synchronized timeline.

### What it cannot prove

Scene, game build, hardware, graphics settings and background load are **unverified**. The importing computer is not assumed to be the capture's computer. Passing checks allows arithmetic observations only. One pair does not establish causation or a confidence interval, even if the reported difference is large. There is no accept/apply decision, holdout or mutation transaction in this report.

Use matching replay, build, settings scope and measurement windows when collecting evidence yourself. Actual automatic tuning still requires the existing controlled experiment and independent confirmation flow, whose live CS2 compatibility remains untested.

Imported report JSON contains metrics, source fingerprints, checks and explicit limits. It omits raw captures, original paths, process IDs and swapchain addresses. Alpha.1 reports without a comparison field continue to load. The source fixtures and browser samples are synthetic and are not real FPS benchmarks.

## Türkçe

**Sonuçlar → İki kaydı karşılaştır** ile önce başlangıç, sonra karşılaştırma CSV dosyasını seç. İki seçimden birini iptal edersen geçmişe yarım rapor eklenmez. Oyun kurulmaz, açılmaz ve ayarlar değiştirilmez. Tarayıcı yalnızca açıkça belirtilmiş sentetik örneği gösterir.

Masaüstü uygulaması raporu yerelde saklar; dışa aktarmada seçtiğin yeni JSON dosyasına yazar. CLI komutu `framelab-cli compare başlangıç.csv karşılaştırma.csv` aynı analizi stdout'a verir; veritabanı veya ayar değişikliği yapmaz.

Yüzde farkı için aynı metrik sütunu, farklı kaynak özeti, iki kayıtta da en az 1.000 kare, en fazla %10 süre farkı ve yükleme/ikincil akış uyarısı olmaması gerekir. Eksik kontrol varsa mutlak değerler görünür, fark hesabı gösterilmez. Bu eşikler yalnızca kalite filtresidir; istatistiksel güven veya aynı sahnede ölçüm garantisi değildir.

FPS ve %1 düşükte artış, kare süresinde azalış daha iyidir. Takılma farkı dakika başına hesaplanır. Grafikler kayıt uzunluğuna göre hizalanır; aynı zaman çizelgesi değildir. Eksik görüntüleme zamanından görüntülenen FPS farkı üretilmez.

**İki dosya FPS kazancını kanıtlamaz.** Sahne, oyun sürümü, donanım ve ayarlar doğrulanmadı. Rapor ayar uygulayamaz. Kaynak yolları, ham kayıt verisi, süreç kimliği ve swapchain adresi JSON'a eklenmez. Gerçek CS2 ve VAC/FACEIT testi yapılmadı; sentetik örnekler performans vaadi değildir.
