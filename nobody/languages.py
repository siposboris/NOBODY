"""Native-script syntax dictionaries for Nobody (experimental, review required).

Native writing systems are preserved. Latin-based profiles use lowercase
words, Vietnamese uses lowercase Vietnamese orthography and accents.
Exactly one selected language per source file.
The words have NOT been professionally reviewed by native speakers.
"""
CANONICAL = ["if","else","elif","while","for","in","def","return","True","False","None","and","or","not","break","continue","let","print","range","len","sum","min","max","abs","round","int","float","str","bool"]
PROFILE_WORDS = {
  "sk": "ak inak inak_ak pokial pre v funkcia vrat pravda nepravda nic a_zaroven alebo nie prerus pokracuj nech vypis rozsah dlzka sucet minimum maximum absolutne zaokruhli cele desatinne text logicke",
  "en": "if else elif while for in function return true false none and or not break continue let print range len sum min max abs round int float str bool",
  "cs": "pokud jinak jinak_pokud dokud pro v funkce vrat pravda nepravda nic a_zaroven nebo ne prerus pokracuj necht vypis rozsah delka soucet minimum maximum abs round int float str bool",
  "de": "wenn sonst sonst_wenn solange fur in funktion zuruck wahr falsch nichts und oder nicht abbruch weiter sei ausgeben bereich lange summe minimum maximum abs round int float str bool",
  "es": "si sino sino_si mientras para en funcion devolver verdadero falso nulo y o no romper continuar dejar imprimir rango longitud suma minimo maximo absoluto redondear entero decimal texto booleano",
  "fr": "si sinon sinon_si tantque pour dans fonction retourner vrai faux rien et ou non arreter continuer soit afficher intervalle longueur somme minimum maximum absolu arrondir entier decimal texte booleen",
  "pt": "se senao senao_se enquanto para em funcao retornar verdadeiro falso nulo e ou nao parar continuar deixe mostrar intervalo comprimento soma minimo maximo absoluto arredondar inteiro decimal texto booleano",
  "ru": "если иначе иначе_если пока для в функция вернуть истина ложь ничего и или не прервать продолжить пусть печать диапазон длина сумма минимум максимум модуль округлить целое дробное строка логическое",
  "zh": "如果 否则 否则如果 当 对于 在 函数 返回 真 假 空 并且 或者 非 跳出 继续 设 打印 范围 长度 求和 最小 最大 绝对值 四舍五入 整数 小数 文本 布尔",
  "hi": "अगर अन्यथा अन्यथा_अगर जबतक केलिए में कार्य लौटाओ सत्य असत्य शून्य और या नहीं तोड़ो जारी मानो छापो सीमा लंबाई योग न्यूनतम अधिकतम निरपेक्ष गोल पूर्णांक दशमलव पाठ तार्किक",
  "ar": "إذا وإلا وإلا_إذا طالما لكل في دالة أرجع صحيح خطأ لاشيء و أو ليس اكسر تابع دع اطبع مدى طول مجموع أصغر أكبر مطلق قرب عدد_صحيح عشري نص منطقي",
  "bn": "যদি নাহলে নাহলে_যদি যতক্ষণ জন্য মধ্যে ফাংশন ফেরত সত্য মিথ্যা শূন্য এবং অথবা নয় থামো চালিয়ে_যাও ধরো ছাপাও পরিসর দৈর্ঘ্য যোগফল সর্বনিম্ন সর্বোচ্চ পরম গোল পূর্ণসংখ্যা দশমিক পাঠ্য যুক্তি",
  "id": "jika lainnya lainnya_jika selama untuk dalam fungsi kembali benar salah kosong dan atau bukan henti lanjut misal cetak rentang panjang jumlah minimum maksimum mutlak bulatkan bilangan desimal teks logika",
  "ur": "اگر ورنہ ورنہ_اگر جبतक کےلیے میں عمل واپس سچ جھوٹ خالی اور یا نہیں توڑو جاری مانو چھاپو حد لمبائی مجموعہ کمترین زیادہترین مطلق گول صحیح_عدد اعشاری متن منطقی",
  "ja": "もし それ以外 それ以外_もし あいだ ため なか 関数 戻す 真 偽 なし かつ または 否定 抜ける 続ける 置く 表示 範囲 長さ 合計 最小 最大 絶対 丸める 整数 小数 文字列 論理",
  "ko": "만약 아니면 아니면_만약 동안 위해 안에 함수 반환 참 거짓 없음 그리고 또는 아님 중단 계속 두기 출력 범위 길이 합계 최소 최대 절댓값 반올림 정수 소수 문자열 논리",
  "tr": "eger degilse degilse_eger iken icin icinde islev dondur dogru yanlis bos ve veya degil kir devam olsun yazdir aralik uzunluk toplam enkucuk enbuyuk mutlak yuvarla tamsayi ondalik metin mantiksal",
  "vi": "nếu nếu_không nếu_không_nếu trong_khi cho trong hàm trả_về đúng sai rỗng và hoặc không dừng tiếp_tục đặt in phạm_vi độ_dài tổng nhỏ_nhất lớn_nhất tuyệt_đối làm_tròn số_nguyên số_thực văn_bản luận_lý",
  "ta": "எனில் இல்லையெனில் இல்லையெனில்_எனில் வரை ஒவ்வொரு இல் செயல்பாடு திருப்பு உண்மை பொய் இல்லை மற்றும் அல்லது அல்ல நிறுத்து தொடர் வை அச்சிடு வரம்பு நீளம் கூட்டுத்தொகை குறைந்தது அதிகபட்சம் முழுமதிப்பு வட்டமிடு முழுஎண் தசமம் உரை தருக்கம்",
  "it": "se altrimenti altrimenti_se mentre per in funzione ritorna vero falso nulla e oppure non interrompi continua sia stampa intervallo lunghezza somma minimo massimo assoluto arrotonda intero decimale testo booleano",
  "th": "ถ้า มิฉะนั้น มิฉะนั้น_ถ้า ขณะที่ สำหรับ ใน ฟังก์ชัน คืนค่า จริง เท็จ ไม่มี และ หรือ ไม่ หยุด ต่อไป ให้ พิมพ์ ช่วง ความยาว ผลรวม ต่ำสุด สูงสุด ค่าสัมบูรณ์ ปัดเศษ จำนวนเต็ม ทศนิยม ข้อความ ตรรกะ",
  "pl": "jezeli inaczej inaczej_jezeli dopoki dla w funkcja zwroc prawda falsz nic oraz lub nie przerwij kontynuuj niech wypisz zakres dlugosc suma minimum maksimum bezwzgledna zaokraglij calkowita dziesietna tekst logiczna"
}
ALIASES = {
    lang: dict(zip(words.split(), CANONICAL))
    for lang, words in PROFILE_WORDS.items()
}
ALIASES["sk"]["kym"] = "while"
ALIASES["en"]["def"] = "def"

# Six more experimental, native-script-friendly syntax vocabularies.
# Keyword naturalness must be reviewed by native speakers.
_ADDED_PROFILES = {
  "he": "אם אחרת אחרת_אם כלעוד עבור בתוך פונקציה החזר אמת שקר ריק וגם או לא עצור המשך הגדר הדפס טווח אורך סכום מינימום מקסימום מוחלט עגל שלם עשרוני טקסט בוליאני",
  "el": "αν αλλιως αλλιως_αν οσο για μεσα συναρτηση επιστροφη αληθες ψευδες κενο και η οχι διακοπη συνεχεια εστω εκτυπωσε ευρος μηκος αθροισμα ελαχιστο μεγιστο απολυτο στρογγυλεψε ακεραιος δεκαδικος κειμενο λογικο",
  "uk": "якщо інакше інакше_якщо поки для в функція повернути істина хиба ніщо і або не перервати продовжити нехай друк діапазон довжина сума мінімум максимум модуль округлити ціле дробове рядок логічне",
  "fa": "اگر وگرنه وگرنه_اگر تاوقتی برای در تابع بازگردان درست نادرست هیچ و یا نه بشکن ادامه بگذار چاپ بازه طول جمع کمینه بیشینه قدرمطلق گردکردن صحیح اعشاری متن منطقی",
  "sw": "ikiwa vinginevyo vinginevyo_ikiwa wakati kwa ndani kazi rudisha kweli si_kweli hakuna na au si vunja endelea acha chapisha masafa urefu jumla kiwango_chini kiwango_juu thamani_kuu zungusha namba_kamili desimali maandishi mantiki",
  "nl": "als anders anders_als zolang voor in functie geef_terug waar onwaar niets en of niet breek verder laat druk bereik lengte som minimum maximum absoluut afronden geheel decimaal tekst logisch"
}
_CANONICAL_NEW = ["if","else","elif","while","for","in","def","return","True","False","None","and","or","not","break","continue","let","print","range","len","sum","min","max","abs","round","int","float","str","bool"]
for _code, _spelling in _ADDED_PROFILES.items():
    _words = _spelling.split()
    if len(_words) != len(_CANONICAL_NEW) or len(_words) != len(set(_words)):
        raise ValueError("Invalid experimental language profile: " + _code)
    ALIASES[_code] = dict(zip(_words, _CANONICAL_NEW))


# Nobody 0.14: additional candidate profiles, pending native-speaker review.
_EXTRA_014 = {"bg":"ако иначе иначе_ако докато за във функция върни вярно невярно нищо и или не прекъсни продължи нека печатай диапазон дължина сума минимум максимум абсолютна закръгли цяло десетично текст логическо","ro":"daca altfel altfel_daca cat_timp pentru in functie returneaza adevarat fals nimic si sau nu intrerupe continua fie afiseaza interval lungime suma minim maxim absolut rotunjeste intreg zecimal text logic","sv":"om annars annars_om medan for inom funktion returnera sant fals inget och eller inte bryt fortsatt lat skriv intervall langd summa minimum maximum absolut avrunda heltal decimaltal text logisk","fi":"jos muuten muuten_jos kunnes varten sisalla funktio palauta tosi epatosi tyhja ja tai ei katkaise jatka olkoon tulosta vali pituus summa minimi maksimi itseisarvo pyorista kokonaisluku desimaali teksti totuusarvo","hy":"եթե այլապես այլապես_եթե քանի_դեռ համար մեջ գործառույթ վերադարձ ճիշտ սխալ ոչինչ և կամ ոչ ընդհատել շարունակել թող տպել միջակայք երկարություն գումար նվազագույն առավելագույն բացարձակ կլորացնել ամբողջ տասնորդական տեքստ տրամաբանական","ka":"თუ სხვაგვარად სხვაგვარად_თუ სანამ ყოველ შიგნით ფუნქცია დააბრუნე ჭეშმარიტი მცდარი არაფერი და ან არა შეწყვიტე გააგრძელე დაე ბეჭდე დიაპაზონი სიგრძე ჯამი მინიმუმი მაქსიმუმი აბსოლუტური დაამრგვალე მთელი ათწილადი ტექსტი ლოგიკური"}
_CANON_014 = "if else elif while for in def return True False None and or not break continue let print range len sum min max abs round int float str bool".split()
for _code, _words in _EXTRA_014.items():
    _keys = _words.split()
    if len(_keys) != len(_CANON_014) or len(set(_keys)) != len(_keys):
        raise ValueError("Invalid 0.14 profile: " + _code)
    ALIASES[_code] = dict(zip(_keys, _CANON_014))

# Nobody 0.15: native-script candidate profiles (terminology not certified).
_MORE_015 = {"ne":"यदि नत्र नत्र_यदि जबसम्म लागि भित्र कार्य फर्काऊ सत्य असत्य शून्य र वा होइन रोक जारी राख छाप दायरा लम्बाइ योग न्यूनतम अधिकतम निरपेक्ष गोलाई पूर्णांक दशमलव पाठ तर्क","mr":"जर अन्यथा अन्यथा_जर जोपर्यंत साठी मध्ये कार्य परत सत्य असत्य शून्य आणि किंवा नाही थांब पुढे चल छापा श्रेणी लांबी बेरीज किमान कमाल निरपेक्ष गोलाकार पूर्णांक दशांश मजकूर तर्क","gu":"જો અન્યથા અન્યથા_જો જ્યાંસુધી માટે માં કાર્ય પરત સાચું ખોટું શૂન્ય અને અથવા નહીં રોકો ચાલુ રાખો છાપો શ્રેણી લંબાઈ સરવાળો ન્યૂનતમ મહત્તમ નિરપેક્ષ ગોળ પૂર્ણાંક દશાંશ લખાણ તર્ક","kn":"ಯದಿ ಇಲ್ಲದಿದ್ದರೆ ಇಲ್ಲದಿದ್ದರೆ_ಯದಿ ತನಕ ಗಾಗಿ ಒಳಗೆ ಕಾರ್ಯ ಹಿಂತಿರುಗಿಸು ಸತ್ಯ ಸುಳ್ಳು ಶೂನ್ಯ ಮತ್ತು ಅಥವಾ ಅಲ್ಲ ನಿಲ್ಲಿಸು ಮುಂದುವರಿಸು ಇಡು ಮುದ್ರಿಸು ವ್ಯಾಪ್ತಿ ಉದ್ದ ಮೊತ್ತ ಕನಿಷ್ಠ ಗರಿಷ್ಠ ನಿರಪೇಕ್ಷ ಸುತ್ತುಗೊಳಿಸು ಪೂರ್ಣಾಂಕ ದಶಮಾಂಶ ಪಠ್ಯ ತಾರ್ಕಿಕ","te":"ఒకవేళ లేకపోతే లేకపోతే_ఒకవేళ వరకు కోసం లో కార్యం తిరిగి నిజం అబద్ధం శూన్యం మరియు లేదా కాదు ఆపు కొనసాగించు ఉంచు ముద్రించు పరిధి పొడవు మొత్తం కనిష్టం గరిష్టం నిరపేక్షం గుండ్రం పూర్ణసంఖ్య దశాంశం పాఠ్యం తార్కికం"}
_CANONICAL_015 = ["if","else","elif","while","for","in","def","return","True","False","None","and","or","not","break","continue","let","print","range","len","sum","min","max","abs","round","int","float","str","bool"]
for _code, _words in _MORE_015.items():
    _spellings = _words.split()
    if len(_spellings) != len(_CANONICAL_015) or len(set(_spellings)) != len(_spellings):
        raise ValueError("Invalid Nobody 0.15 syntax profile: "+_code)
    ALIASES[_code] = dict(zip(_spellings, _CANONICAL_015))
# The AI profile is a structured canonical syntax, not a claim of AI sentience.
ALIASES["ai"] = {"ai_"+name.lower(): name for name in _CANONICAL_015}

PREFERRED = {}
for lang, aliases in ALIASES.items():
    preferred = {}
    for spelling, canonical in aliases.items():
        preferred.setdefault(canonical, spelling)
    PREFERRED[lang] = preferred
LANGUAGE_NAMES = {
  "sk": "Slovak",
  "en": "English",
  "cs": "Czech",
  "de": "German",
  "es": "Spanish",
  "fr": "French",
  "pt": "Portuguese",
  "ru": "Russian",
  "zh": "Mandarin Chinese",
  "hi": "Hindi",
  "ar": "Arabic",
  "bn": "Bengali",
  "id": "Indonesian",
  "ur": "Urdu",
  "ja": "Japanese",
  "ko": "Korean",
  "tr": "Turkish",
  "vi": "Vietnamese",
  "ta": "Tamil",
  "it": "Italian",
  "th": "Thai",
  "pl": "Polish"
}
NATIVE_SCRIPT_PROFILES = ("ru", "zh", "hi", "ar", "bn", "ur", "ja", "ko", "ta", "th", "vi")

LANGUAGE_NAMES.update({"he":"Hebrew","el":"Greek","uk":"Ukrainian","fa":"Persian","sw":"Swahili","nl":"Dutch"})
NATIVE_SCRIPT_PROFILES = tuple(NATIVE_SCRIPT_PROFILES) + ("he","el","uk","fa")

LANGUAGE_NAMES.update({"bg":"Bulgarian","ro":"Romanian","sv":"Swedish","fi":"Finnish","hy":"Armenian","ka":"Georgian"})
NATIVE_SCRIPT_PROFILES = tuple(NATIVE_SCRIPT_PROFILES) + ("bg","hy","ka")

LANGUAGE_NAMES.update({"ne":"Nepali","mr":"Marathi","gu":"Gujarati","kn":"Kannada","te":"Telugu","ai":"AI canonical syntax"})
NATIVE_SCRIPT_PROFILES = tuple(NATIVE_SCRIPT_PROFILES) + ("ne","mr","gu","kn","te")
