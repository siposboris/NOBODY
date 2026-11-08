//! Native Nobody syntax profiles. Data ported from the historical Python reference.
//! This data is executable-independent: Rust does not import Python at runtime.
//! Word choice is experimental and still needs native-speaker reviews.
//! The `ai` profile intentionally uses familiar code keywords, not `ai_` prefixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Profile {
    pub code: &'static str,
    pub words: &'static str,
}
pub const CANONICAL: [&str; 29] = [
    "if", "else", "elif", "while", "for", "in", "def", "return", "true", "false", "none", "and",
    "or", "not", "break", "continue", "let", "print", "range", "len", "sum", "min", "max", "abs",
    "round", "int", "float", "str", "bool",
];
pub const PROFILES: [Profile;40] = [
    Profile { code: "ai", words: "if else elif while for in fn return true false null and or not break continue let print range len sum min max abs round int float str bool" },
    Profile { code: "ar", words: "إذا وإلا وإلا_إذا طالما لكل في دالة أرجع صحيح خطأ لاشيء و أو ليس اكسر تابع دع اطبع مدى طول مجموع أصغر أكبر مطلق قرب عدد_صحيح عشري نص منطقي" },
    Profile { code: "bg", words: "ако иначе иначе_ако докато за във функция върни вярно невярно нищо и или не прекъсни продължи нека печатай диапазон дължина сума минимум максимум абсолютна закръгли цяло десетично текст логическо" },
    Profile { code: "bn", words: "যদি নাহলে নাহলে_যদি যতক্ষণ জন্য মধ্যে ফাংশন ফেরত সত্য মিথ্যা শূন্য এবং অথবা নয় থামো চালিয়ে_যাও ধরো ছাপাও পরিসর দৈর্ঘ্য যোগফল সর্বনিম্ন সর্বোচ্চ পরম গোল পূর্ণসংখ্যা দশমিক পাঠ্য যুক্তি" },
    Profile { code: "cs", words: "pokud jinak jinak_pokud dokud pro v funkce vrat pravda nepravda nic a_zaroven nebo ne prerus pokracuj necht vypis rozsah delka soucet minimum maximum abs round int float str bool" },
    Profile { code: "de", words: "wenn sonst sonst_wenn solange fur in funktion zuruck wahr falsch nichts und oder nicht abbruch weiter sei ausgeben bereich lange summe minimum maximum abs round int float str bool" },
    Profile { code: "el", words: "αν αλλιως αλλιως_αν οσο για μεσα συναρτηση επιστροφη αληθες ψευδες κενο και η οχι διακοπη συνεχεια εστω εκτυπωσε ευρος μηκος αθροισμα ελαχιστο μεγιστο απολυτο στρογγυλεψε ακεραιος δεκαδικος κειμενο λογικο" },
    Profile { code: "en", words: "if else elif while for in function return true false none and or not break continue let print range len sum min max abs round int float str bool" },
    Profile { code: "es", words: "si sino sino_si mientras para en funcion devolver verdadero falso nulo y o no romper continuar dejar imprimir rango longitud suma minimo maximo absoluto redondear entero decimal texto booleano" },
    Profile { code: "fa", words: "اگر وگرنه وگرنه_اگر تاوقتی برای در تابع بازگردان درست نادرست هیچ و یا نه بشکن ادامه بگذار چاپ بازه طول جمع کمینه بیشینه قدرمطلق گردکردن صحیح اعشاری متن منطقی" },
    Profile { code: "fi", words: "jos muuten muuten_jos kunnes varten sisalla funktio palauta tosi epatosi tyhja ja tai ei katkaise jatka olkoon tulosta vali pituus summa minimi maksimi itseisarvo pyorista kokonaisluku desimaali teksti totuusarvo" },
    Profile { code: "fr", words: "si sinon sinon_si tantque pour dans fonction retourner vrai faux rien et ou non arreter continuer soit afficher intervalle longueur somme minimum maximum absolu arrondir entier decimal texte booleen" },
    Profile { code: "gu", words: "જો અન્યથા અન્યથા_જો જ્યાંસુધી માટે માં કાર્ય પરત સાચું ખોટું શૂન્ય અને અથવા નહીં રોકો ચાલુ રાખો છાપો શ્રેણી લંબાઈ સરવાળો ન્યૂનતમ મહત્તમ નિરપેક્ષ ગોળ પૂર્ણાંક દશાંશ લખાણ તર્ક" },
    Profile { code: "he", words: "אם אחרת אחרת_אם כלעוד עבור בתוך פונקציה החזר אמת שקר ריק וגם או לא עצור המשך הגדר הדפס טווח אורך סכום מינימום מקסימום מוחלט עגל שלם עשרוני טקסט בוליאני" },
    Profile { code: "hi", words: "अगर अन्यथा अन्यथा_अगर जबतक केलिए में कार्य लौटाओ सत्य असत्य शून्य और या नहीं तोड़ो जारी मानो छापो सीमा लंबाई योग न्यूनतम अधिकतम निरपेक्ष गोल पूर्णांक दशमलव पाठ तार्किक" },
    Profile { code: "hy", words: "եթե այլապես այլապես_եթե քանի_դեռ համար մեջ գործառույթ վերադարձ ճիշտ սխալ ոչինչ և կամ ոչ ընդհատել շարունակել թող տպել միջակայք երկարություն գումար նվազագույն առավելագույն բացարձակ կլորացնել ամբողջ տասնորդական տեքստ տրամաբանական" },
    Profile { code: "id", words: "jika lainnya lainnya_jika selama untuk dalam fungsi kembali benar salah kosong dan atau bukan henti lanjut misal cetak rentang panjang jumlah minimum maksimum mutlak bulatkan bilangan desimal teks logika" },
    Profile { code: "it", words: "se altrimenti altrimenti_se mentre per in funzione ritorna vero falso nulla e oppure non interrompi continua sia stampa intervallo lunghezza somma minimo massimo assoluto arrotonda intero decimale testo booleano" },
    Profile { code: "ja", words: "もし それ以外 それ以外_もし あいだ ため なか 関数 戻す 真 偽 なし かつ または 否定 抜ける 続ける 置く 表示 範囲 長さ 合計 最小 最大 絶対 丸める 整数 小数 文字列 論理" },
    Profile { code: "ka", words: "თუ სხვაგვარად სხვაგვარად_თუ სანამ ყოველ შიგნით ფუნქცია დააბრუნე ჭეშმარიტი მცდარი არაფერი და ან არა შეწყვიტე გააგრძელე დაე ბეჭდე დიაპაზონი სიგრძე ჯამი მინიმუმი მაქსიმუმი აბსოლუტური დაამრგვალე მთელი ათწილადი ტექსტი ლოგიკური" },
    Profile { code: "kn", words: "ಯದಿ ಇಲ್ಲದಿದ್ದರೆ ಇಲ್ಲದಿದ್ದರೆ_ಯದಿ ತನಕ ಗಾಗಿ ಒಳಗೆ ಕಾರ್ಯ ಹಿಂತಿರುಗಿಸು ಸತ್ಯ ಸುಳ್ಳು ಶೂನ್ಯ ಮತ್ತು ಅಥವಾ ಅಲ್ಲ ನಿಲ್ಲಿಸು ಮುಂದುವರಿಸು ಇಡು ಮುದ್ರಿಸು ವ್ಯಾಪ್ತಿ ಉದ್ದ ಮೊತ್ತ ಕನಿಷ್ಠ ಗರಿಷ್ಠ ನಿರಪೇಕ್ಷ ಸುತ್ತುಗೊಳಿಸು ಪೂರ್ಣಾಂಕ ದಶಮಾಂಶ ಪಠ್ಯ ತಾರ್ಕಿಕ" },
    Profile { code: "ko", words: "만약 아니면 아니면_만약 동안 위해 안에 함수 반환 참 거짓 없음 그리고 또는 아님 중단 계속 두기 출력 범위 길이 합계 최소 최대 절댓값 반올림 정수 소수 문자열 논리" },
    Profile { code: "mr", words: "जर अन्यथा अन्यथा_जर जोपर्यंत साठी मध्ये कार्य परत सत्य असत्य शून्य आणि किंवा नाही थांब पुढे चल छापा श्रेणी लांबी बेरीज किमान कमाल निरपेक्ष गोलाकार पूर्णांक दशांश मजकूर तर्क" },
    Profile { code: "ne", words: "यदि नत्र नत्र_यदि जबसम्म लागि भित्र कार्य फर्काऊ सत्य असत्य शून्य र वा होइन रोक जारी राख छाप दायरा लम्बाइ योग न्यूनतम अधिकतम निरपेक्ष गोलाई पूर्णांक दशमलव पाठ तर्क" },
    Profile { code: "nl", words: "als anders anders_als zolang voor in functie geef_terug waar onwaar niets en of niet breek verder laat druk bereik lengte som minimum maximum absoluut afronden geheel decimaal tekst logisch" },
    Profile { code: "pl", words: "jezeli inaczej inaczej_jezeli dopoki dla w funkcja zwroc prawda falsz nic oraz lub nie przerwij kontynuuj niech wypisz zakres dlugosc suma minimum maksimum bezwzgledna zaokraglij calkowita dziesietna tekst logiczna" },
    Profile { code: "pt", words: "se senao senao_se enquanto para em funcao retornar verdadeiro falso nulo e ou nao parar continuar deixe mostrar intervalo comprimento soma minimo maximo absoluto arredondar inteiro decimal texto booleano" },
    Profile { code: "ro", words: "daca altfel altfel_daca cat_timp pentru in functie returneaza adevarat fals nimic si sau nu intrerupe continua fie afiseaza interval lungime suma minim maxim absolut rotunjeste intreg zecimal text logic" },
    Profile { code: "ru", words: "если иначе иначе_если пока для в функция вернуть истина ложь ничего и или не прервать продолжить пусть печать диапазон длина сумма минимум максимум модуль округлить целое дробное строка логическое" },
    Profile { code: "sk", words: "ak inak inak_ak pokial pre v funkcia vrat pravda nepravda nic a_zaroven alebo nie prerus pokracuj nech vypis rozsah dlzka sucet minimum maximum absolutne zaokruhli cele desatinne text logicke" },
    Profile { code: "sv", words: "om annars annars_om medan for inom funktion returnera sant fals inget och eller inte bryt fortsatt lat skriv intervall langd summa minimum maximum absolut avrunda heltal decimaltal text logisk" },
    Profile { code: "sw", words: "ikiwa vinginevyo vinginevyo_ikiwa wakati kwa ndani kazi rudisha kweli si_kweli hakuna na au si vunja endelea acha chapisha masafa urefu jumla kiwango_chini kiwango_juu thamani_kuu zungusha namba_kamili desimali maandishi mantiki" },
    Profile { code: "ta", words: "எனில் இல்லையெனில் இல்லையெனில்_எனில் வரை ஒவ்வொரு இல் செயல்பாடு திருப்பு உண்மை பொய் இல்லை மற்றும் அல்லது அல்ல நிறுத்து தொடர் வை அச்சிடு வரம்பு நீளம் கூட்டுத்தொகை குறைந்தது அதிகபட்சம் முழுமதிப்பு வட்டமிடு முழுஎண் தசமம் உரை தருக்கம்" },
    Profile { code: "te", words: "ఒకవేళ లేకపోతే లేకపోతే_ఒకవేళ వరకు కోసం లో కార్యం తిరిగి నిజం అబద్ధం శూన్యం మరియు లేదా కాదు ఆపు కొనసాగించు ఉంచు ముద్రించు పరిధి పొడవు మొత్తం కనిష్టం గరిష్టం నిరపేక్షం గుండ్రం పూర్ణసంఖ్య దశాంశం పాఠ్యం తార్కికం" },
    Profile { code: "th", words: "ถ้า มิฉะนั้น มิฉะนั้น_ถ้า ขณะที่ สำหรับ ใน ฟังก์ชัน คืนค่า จริง เท็จ ไม่มี และ หรือ ไม่ หยุด ต่อไป ให้ พิมพ์ ช่วง ความยาว ผลรวม ต่ำสุด สูงสุด ค่าสัมบูรณ์ ปัดเศษ จำนวนเต็ม ทศนิยม ข้อความ ตรรกะ" },
    Profile { code: "tr", words: "eger degilse degilse_eger iken icin icinde islev dondur dogru yanlis bos ve veya degil kir devam olsun yazdir aralik uzunluk toplam enkucuk enbuyuk mutlak yuvarla tamsayi ondalik metin mantiksal" },
    Profile { code: "uk", words: "якщо інакше інакше_якщо поки для в функція повернути істина хиба ніщо і або не перервати продовжити нехай друк діапазон довжина сума мінімум максимум модуль округлити ціле дробове рядок логічне" },
    Profile { code: "ur", words: "اگر ورنہ ورنہ_اگر جبतक کےلیے میں عمل واپس سچ جھوٹ خالی اور یا نہیں توڑو جاری مانو چھاپو حد لمبائی مجموعہ کمترین زیادہترین مطلق گول صحیح_عدد اعشاری متن منطقی" },
    Profile { code: "vi", words: "nếu nếu_không nếu_không_nếu trong_khi cho trong hàm trả_về đúng sai rỗng và hoặc không dừng tiếp_tục đặt in phạm_vi độ_dài tổng nhỏ_nhất lớn_nhất tuyệt_đối làm_tròn số_nguyên số_thực văn_bản luận_lý" },
    Profile { code: "zh", words: "如果 否则 否则如果 当 对于 在 函数 返回 真 假 空 并且 或者 非 跳出 继续 设 打印 范围 长度 求和 最小 最大 绝对值 四舍五入 整数 小数 文本 布尔" },
];
pub fn profile(code: &str) -> Option<&'static Profile> {
    PROFILES.iter().find(|p| p.code == code)
}
pub fn word(code: &str, semantic: &str) -> Option<&'static str> {
    let p = profile(code)?;
    let index = CANONICAL.iter().position(|&x| x == semantic)?;
    p.words.split_whitespace().nth(index)
}
pub fn semantic(code: &str, spelling: &str) -> Option<&'static str> {
    let p = profile(code)?;
    p.words
        .split_whitespace()
        .position(|x| x == spelling)
        .map(|index| CANONICAL[index])
}
/// Editor display metadata only. Source bytes remain in logical UTF-8 order.
/// Returns None for unavailable profiles; bidi control characters remain
/// forbidden by the Rust lexer regardless of the profile direction.
pub fn direction(code: &str) -> Option<&'static str> {
    profile(code)?;
    Some(if matches!(code, "ar" | "ur" | "he" | "fa") {
        "rtl"
    } else {
        "ltr"
    })
}
/// Validate the 40 *implemented* syntax dictionaries without Python.
/// This is a structural audit, not a native-speaker terminology review.
pub fn audit_profiles() -> Vec<String> {
    use std::collections::BTreeSet;
    use unicode_normalization::UnicodeNormalization;

    let mut errors = Vec::new();
    let mut seen_codes = BTreeSet::new();
    for profile in PROFILES {
        if !seen_codes.insert(profile.code) {
            errors.push(format!("{}: duplicate profile code", profile.code));
        }
        let words: Vec<_> = profile.words.split_whitespace().collect();
        if words.len() != CANONICAL.len() {
            errors.push(format!(
                "{}: expected {} keywords, got {}",
                profile.code,
                CANONICAL.len(),
                words.len()
            ));
        }
        let mut seen_words = BTreeSet::new();
        for word in words {
            if !seen_words.insert(word) {
                errors.push(format!("{}: duplicate keyword {word:?}", profile.code));
            }
            if word.is_empty()
                || word
                    .chars()
                    .any(|ch| ch.is_uppercase() || ch.is_control() || ch.is_whitespace())
            {
                errors.push(format!("{}: invalid keyword {word:?}", profile.code));
            }
            if word.nfc().collect::<String>() != word {
                errors.push(format!("{}: non-NFC keyword {word:?}", profile.code));
            }
            if word.chars().any(|ch| {
                matches!(
                    ch as u32,
                    0x061C | 0x200E | 0x200F | 0x202A..=0x202E | 0x2066..=0x2069
                )
            }) {
                errors.push(format!(
                    "{}: forbidden bidi control in keyword",
                    profile.code
                ));
            }
        }
    }
    errors
}
pub fn is_reserved(code: &str, spelling: &str) -> bool {
    semantic(code, spelling).is_some()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_profiles_have_unique_full_vocab() {
        assert_eq!(PROFILES.len(), 40);
        for p in PROFILES {
            let terms: Vec<_> = p.words.split_whitespace().collect();
            assert_eq!(terms.len(), CANONICAL.len(), "{}", p.code);
            let mut sorted = terms.clone();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), terms.len(), "{}", p.code);
        }
    }
    #[test]
    fn profile_audit_reports_no_structural_errors() {
        assert!(audit_profiles().is_empty(), "{:?}", audit_profiles());
    }
    #[test]
    fn ai_syntax_is_conventional() {
        assert_eq!(word("ai", "def"), Some("fn"));
        assert_eq!(word("ai", "let"), Some("let"));
        assert_eq!(word("ai", "true"), Some("true"));
        assert_eq!(word("ai", "none"), Some("null"));
        assert!(semantic("ai", "ai_if").is_none());
    }
    #[test]
    fn direction_matches_reference_for_all_profiles() {
        for profile in PROFILES {
            let expected = if matches!(profile.code, "ar" | "ur" | "he" | "fa") {
                "rtl"
            } else {
                "ltr"
            };
            assert_eq!(direction(profile.code), Some(expected));
        }
        assert_eq!(direction("unknown"), None);
    }
    #[test]
    fn native_script_words_are_preserved() {
        assert_eq!(word("ar", "if"), Some("إذا"));
        assert_eq!(word("ja", "if"), Some("もし"));
        assert_eq!(word("he", "if"), Some("אם"));
    }
}
