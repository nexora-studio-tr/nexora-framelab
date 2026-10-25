# Nexora FrameLab — CS2 ürün ve mimari kararı

Durum: 0.1.0-alpha.1 uygulandı; aşağıdaki metin ilk ürün kararını kaydeder.
Gerçekleşen ilk sürüm kapsamı: en fazla iki grafik adayı, beş eşlenik koşu ve bağımsız doğrulama; yalnızca tek kazanan aday uygulanır. Güç planı ve kalıcı FPS sınırı değişiklikleri ertelendi. Desteklenen ayar şeması `Version 16` ile sınırlıdır; diğer sürümler salt okunur. Canlı CS2 testi kullanıcının isteğiyle yapılmadı. Güncel doğrulama durumu [VALIDATION](docs/VALIDATION.md) içindedir.
Tarih: 25 Ekim 2026.

## Ürün

Windows 11 x64 üzerinde CS2 için donanımı otomatik tanıyan, mevcut performansı ölçen, uygun ayarları kontrollü olarak deneyen ve faydası doğrulanan profili uygulayan açık kaynak masaüstü uygulaması.

Hedef kullanıcı: kendi bilgisayarında CS2 oynayan ve genel bir ayar listesini elle denemek istemeyen oyuncu. İlk sürüm CS2 ile sınırlıdır; diğer oyunlar sonraki adaptörlerdir. Sabit bir FPS artışı vaat edilmez. Fayda bulunmadığında sonuç açıkça gösterilir ve başlangıç ayarları korunur.

## Kullanıcı akışı

1. Uygulama bilgisayarı, Steam/CS2 kurulumunu ve desteklenen ayar sürümünü bulur.
2. Kullanıcı bir kez görüntü hedefini seçer: **Dengeli** (varsayılan), **Rekabetçi** veya **Görüntüyü koru**. Değiştirilebilecek ayarlar önceden gösterilir.
3. **Analiz et ve optimize et** ile kontrollü yerel test başlar. Mevcut maçta deney yapılmaz. Araçtan başlatılan test oturumu kullanılır; CS2 zaten açıksa oturumun kapatılması istenir.
4. Başlangıç ölçümü alınır. Donanım özellikleri ve ölçümler birlikte değerlendirilerek sınırlı sayıda aday ayar seçilir.
5. Mevcut durum yedeklenir; bir seferde tek ayar veya gerekçesi tanımlı küçük bir ayar grubu denenir.
6. Aynı test sahnesinde önce/sonra karşılaştırması yapılır. Kazancı doğrulanmayan veya takılmayı artıran değişiklik geri alınır.
7. Sonuç panelinde uygulanan değişiklik, ölçülen kazanç, belirsizlik ve görüntü kalitesinden verilen ödün açıklanır. **Eski ayarlara dön** her zaman erişilebilirdir.

## Otomatik tanıma ve ölçüm

| Alan | Toplanacak bilgi ve davranış |
| --- | --- |
| CPU | Model, fiziksel/mantıksal çekirdekler, işlemci yükü; yalnızca model adına göre karar verilmez |
| GPU | Tüm adaptörler, etkin oyun adaptörü, sürücü, ayrılmış/paylaşılan bellek; iGPU ve çoklu GPU ayrı ele alınır |
| RAM | Kapasite, kullanılabilir bellek ve bellek baskısı; RAM boşaltma gösterisi yapılmaz |
| Disk | Oyun konumu, depolama türü bulunabiliyorsa türü, kullanılabilir alan |
| Ekran ve güç | Çözünürlük, yenileme hızı, pencere modu, güç kaynağı/planı |
| Oyun | Steam kütüphaneleri, CS2 sürümü, ilgili yerel ayar profili; belirsiz çoklu hesapta seçim istenir |
| Kare ölçümü | Sunulan/gösterilen kare metrikleri ayrılır; FPS, %1 düşük FPS, p95/p99 kare süresi, takılma sayısı ve veri kaybı kaydedilir |
| Ek sensörler | Güç/sıcaklık yalnızca desteklenen kaynaktan alınabiliyorsa gösterilir; bulunamayan veri uydurulmaz |

Donanım bilgisi adayları daraltır; CPU/GPU kullanım yüzdesi tek başına kesin darboğaz kanıtı sayılmaz. Veri yetersizse sistem kesin teşhis veya otomatik müdahale üretmez.

## Otomatik müdahalenin sınırı

- Sürümü doğrulanan CS2 grafik seçenekleri ve FPS sınırı, seçilen görüntü hedefine göre denenir. Dengeli modda çözünürlük kendiliğinden düşürülmez.
- Windows güç planı gibi oturumluk değişiklikler yalnızca desteklenen cihazlarda, önceden izin verilen kapsamda ve ölçümle denenir; mevcut değer oyun/uygulama çıkışında geri yüklenir. Güvenilir gözlem yapılamayan güç deneyi atlanır.
- Sürücü durumu ve arka plan yükü teşhis edilir. İlk sürüm sürücü yüklemez, kullanıcının uygulamalarını zorla kapatmaz veya BIOS/overclock ayarı yapmaz.
- Güvenlik ve anti-cheat ayarları değiştirilmez. Oyun sürecine DLL eklenmez veya oyun belleğine yazılmaz. Bunlar bir VAC/FACEIT uyumluluk garantisi değildir; destek durumu gerçek testle raporlanır.
- Ayar dosyaları CS2 kapalıyken alan bazında güncellenir. Steam Cloud veya kullanıcı aynı dosyayı değiştirirse çakışma algılanır; yeni veri körlemesine ezilmez.

## Deney motoru: farklılaşmanın merkezi

Uygulama bir donanım adına hazır paket uygulamak yerine o bilgisayarda kontrollü deney yürütür. Aynı sahne/replay, süre, görüntü hedefi ve ölçüm koşulları korunur; ayar değişiklikleri, sahne değişimiyle karıştırılmaz. Menü, yükleme ve eksik yakalama içeren koşular geçersiz olarak işaretlenir.

Kısa ön taramadan sonra nihai profil en az beş geçerli eşlenik karşılaştırmayla doğrulanır. Isınma ve test sırası etkisi için A/B sırası dengelenir; belirsizlik kareleri bağımsız örnek sayarak yapay biçimde küçültülmez. Karar koşu düzeyindeki sonuçlara dayanır. Süre tahmini başlamadan gösterilir; kullanıcı deneyi durdurabilir.

%1 düşük FPS tanımı raporda açık olur: en yavaş %1 kare süresinin ortalamasının tersi. Sunulan ve gösterilen FPS farklı adlarla sunulur. Kazanç, ölçüm gürültüsünden ayırt edilemiyorsa **kanıtlanmadı** sayılır. Bir ayarın faydası, son birleşik profilin tekrar test edilmesi gereğini ortadan kaldırmaz.

## Mimari

| Bileşen | Karar |
| --- | --- |
| Masaüstü kabuğu | Tauri 2, React, TypeScript; erişilebilir Radix bileşenleri, Türkçe/İngilizce arayüz |
| Yerel çekirdek | Rust; donanım keşfi, deney durum makinesi, CS2 adaptörü ve geri alma |
| Donanım kaynakları | Windows API/WMI ve DXGI; VRAM için ayrılmış ve paylaşılan bellek ayrı tutulur |
| Kare yakalama | Lisansı ve sürümü sabitlenen PresentMon/ETW; düşük örnekleme yükü ve kayıp olay kontrolü |
| Kayıt | Yerel SQLite: deneyler, uyumluluk, değişiklik günlüğü, yedek özetleri; büyük ham yakalamalar sınırlı ayrı dosyalarda |
| Yetki | Arayüz normal kullanıcıyla çalışır; gerekirse yalnızca tanımlı işlemler için dar yetkili yardımcı; genel komut çalıştırma arayüzü yoktur |
| Profil motoru | Sürümlü, incelenebilir kural ve adaptörler; internetten indirilen komut/script çalıştırılmaz |
| Hesap ve internet | İlk sürüm hesap, sunucu veya ücretli API gerektirmez. Veriler yereldir; rapor dışa aktarma kullanıcı eylemidir |
| Lisans | Yeni proje kodu için MIT hedeflenir; bağımlılık bildirimleri ayrıca tutulur |

## Geri alma ve hata davranışı

Her müdahale için önce eski değer ve dosya özeti kalıcı günlüğe yazılır; ardından atomik değişiklik yapılır ve yeni durum doğrulanır. Elektrik kesilmesi, yarıda kalan deney ve uygulama çökmesi sonraki açılışta tespit edilir. Kullanıcı sonradan aynı ayarı değiştirmişse geri alma da bu değişikliği körlemesine ezmez. Disk yetersizliği veya yedekleme hatasında müdahale başlamaz.

## Arayüz

Üç ana görünüm: **Sistemim**, **Optimize et**, **Sonuçlar**. Donanım kartlarının altında ölçümün ne söylediği ve hangi ayarın neden deneneceği kısa biçimde gösterilir. Canlı grafikte FPS ve kare süresi, sonuçta önce/sonra karşılaştırması bulunur. Teknik günlük isteğe bağlı açılır; temel akış jargon gerektirmez. Geri alma ve durdurma sürekli görünürdür.

## Geliştirme sırası ve kullanılabilir alfa kapısı

1. Donanım keşfi, CS2 bulma, salt okunur tanı paneli ve kayıtlı demo.
2. Gerçek kare yakalama, veri doğrulama ve tekrarlanabilir yerel benchmark.
3. CS2 adaptörü, yedekleme, çakışma algılama ve çökme sonrası geri alma.
4. Sınırlı otomatik deney, aday seçimi ve birleşik profil doğrulaması.
5. Profesyonel iki dilli arayüz, kurulum paketi, dokümantasyon ve gerçek donanım testleri.

Alfa için: en az bir gerçek CS2/donanım senaryosu uçtan uca doğrulanmalı; desteklenmeyen sensör ve adaptörler açıkça belirtilmeli; simülasyon donanım testi gibi gösterilmemelidir. Daha geniş NVIDIA/AMD/Intel desteği için ayrı gerçek test matrisi yayımlanır. Yakalama ek yükü ve uygulamanın kendi bellek/CPU kullanımı da ölçülür.

Anlamlı testler: donanım sağlayıcılarının eksik/çoklu adaptör verileri, yakalama kaybı, istatistik ve metrik tanımları, değişen ayar dosyası, yarım yazma, disk/yetki hatası, çökme sonrası kurtarma, profil birleşiminde gerileme ve arayüzün durdurma/geri alma akışı. Gerçek FPS kazancı yalnızca gerçek CS2 deneyinden raporlanır.

## Bakım yükü

Yüksektir: CS2 güncellemeleri, ayar şemaları, Windows/sürücü değişiklikleri ve donanım çeşitliliği uyumluluk takibi gerektirir. Bu nedenle ilk kapsam tek oyun, sınırlı müdahale ve sürümlü adaptördür. Daha fazla oyun, bulut topluluğu, sürücü yönetimi ve özel kernel bileşenleri ilk sürüme eklenmez.

## Birincil teknik kaynaklar

- [PresentMon kaynak ve kapsamı](https://github.com/GameTechDev/PresentMon)
- [PresentMon ölçüm tanımları](https://github.com/GameTechDev/PresentMon/blob/main/README-CaptureApplication.md)
- [Microsoft DXGI adaptör/bellek açıklaması](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_6/ns-dxgi1_6-dxgi_adapter_desc3)
- [Valve CS2 Trusted Mode](https://help.steampowered.com/en/faqs/view/09a0-4879-4353-ef95)

Kaynaklar teknik sınırları destekler; bu planın uygulanmış veya canlı testten geçmiş olduğu anlamına gelmez.
