use crate::model::enums::Era;
use cow_utils::CowUtils;
use lazy_static::lazy_static;
use std::borrow::Cow;
use std::collections::HashMap;

pub struct Nations;
impl Nations {
    pub fn from_id(id: u32) -> Option<StaticNation> {
        from_id_from_map(id, Era::Early, &EA_NATIONS_BY_ID)
            .or_else(|| from_id_from_map(id, Era::Middle, &MA_NATIONS_BY_ID))
            .or_else(|| from_id_from_map(id, Era::Late, &LA_NATIONS_BY_ID))
    }

    pub fn from_id_6(id: u32) -> Option<StaticNation> {
        from_id_from_map(id, Era::Early, &DOM_6_EA_BY_ID)
            .or_else(|| from_id_from_map(id, Era::Middle, &DOM_6_MA_BY_ID))
            .or_else(|| from_id_from_map(id, Era::Late, &DOM_6_LA_BY_ID))
    }

    pub fn from_name_prefix(
        name_prefix: &str,
        option_era_filter: Option<Era>,
    ) -> Vec<StaticNation> {
        // todo: cow utils
        let name_prefix = name_prefix.to_owned().to_lowercase().into();

        // okay we want to try with this era, and if it doesn't work we forget about it and try that
        let (name_prefix, option_specific_era) = extract_possible_nation_prefix(name_prefix);
        let sanitised_prefix = sanitise_text(name_prefix);

        let option_era = option_specific_era.or(option_era_filter);

        let option_nations_by_id: Option<(&'static HashMap<u32, &'static str>, Era)> =
            match option_era {
                Some(Era::Early) => Some((&EA_NATIONS_BY_ID, Era::Early)),
                Some(Era::Middle) => Some((&MA_NATIONS_BY_ID, Era::Middle)),
                Some(Era::Late) => Some((&LA_NATIONS_BY_ID, Era::Late)),
                None => None,
            };

        match option_nations_by_id {
            Some((nations_by_id, era)) => {
                find_nation_options_from_map(nations_by_id, sanitised_prefix.as_ref(), era)
            }
            // guess we just have to look in all 3
            None => {
                find_nation_options_from_map(
                    &EA_NATIONS_BY_ID,
                    sanitised_prefix.as_ref(),
                    Era::Early,
                )
            }
            .or_else(|| {
                find_nation_options_from_map(
                    &MA_NATIONS_BY_ID,
                    sanitised_prefix.as_ref(),
                    Era::Middle,
                )
            })
            .or_else(|| {
                find_nation_options_from_map(
                    &LA_NATIONS_BY_ID,
                    sanitised_prefix.as_ref(),
                    Era::Late,
                )
            }),
        }
        .unwrap_or_default()
    }

    pub fn from_name_prefix_6(
        name_prefix: &str,
        option_era_filter: Option<Era>,
    ) -> Vec<StaticNation> {
        let name_prefix = sanitise_text(name_prefix.to_owned().to_lowercase().into());
        let (name_prefix, option_specific_era) = extract_possible_nation_prefix(name_prefix);
        let option_era = option_specific_era.or(option_era_filter);

        match option_era {
            Some(Era::Early) => {
                find_nation_options_from_map_6(&DOM_6_EA_BY_NAME, name_prefix.as_ref(), Era::Early)
            }
            Some(Era::Middle) => {
                find_nation_options_from_map_6(&DOM_6_MA_BY_NAME, name_prefix.as_ref(), Era::Middle)
            }
            Some(Era::Late) => {
                find_nation_options_from_map_6(&DOM_6_LA_BY_NAME, name_prefix.as_ref(), Era::Late)
            }
            None => find_nation_options_no_era_6(name_prefix.as_ref()),
        }
    }
}

fn extract_possible_nation_prefix(lowercase_name_prefix: Cow<str>) -> (Cow<str>, Option<Era>) {
    if lowercase_name_prefix.starts_with("ea ") {
        (cow_drop(lowercase_name_prefix, 3), Some(Era::Early))
    } else if lowercase_name_prefix.starts_with("ma ") {
        (cow_drop(lowercase_name_prefix, 3), Some(Era::Middle))
    } else if lowercase_name_prefix.starts_with("la ") {
        (cow_drop(lowercase_name_prefix, 3), Some(Era::Late))
    } else {
        (lowercase_name_prefix, None)
    }
}

fn sanitise_text(mut lowercase_text: Cow<str>) -> Cow<str> {
    lowercase_text = cow_r_c(lowercase_text, '\'', "");
    lowercase_text = cow_r_c(lowercase_text, ',', "");
    lowercase_text = cow_r_c(lowercase_text, 'è', "e");
    lowercase_text = cow_r_c(lowercase_text, 'å', "a");
    lowercase_text = cow_r_c(lowercase_text, '-', "");
    lowercase_text
}

// should save ourselves an allocation in the borrowed -> borrowed cases.
// Does this matter? absolutely not.
fn cow_r_c<'a>(x: Cow<'a, str>, from: char, to: &str) -> Cow<'a, str> {
    match x {
        Cow::Owned(owned) => Cow::Owned(owned.replace(from, to)),
        Cow::Borrowed(borrowed) => borrowed.cow_replace(from, to),
    }
}
fn cow_r_s<'a>(x: Cow<'a, str>, from: &str, to: &str) -> Cow<'a, str> {
    match x {
        Cow::Owned(owned) => Cow::Owned(owned.replace(from, to)),
        Cow::Borrowed(borrowed) => borrowed.cow_replace(from, to),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StaticNation {
    pub id: u32,
    pub name: &'static str,
    pub era: Era,
}

lazy_static! {
    pub static ref EA_NATIONS_BY_ID: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(5, "Arcoscephale");
        m.insert(6, "Ermor");
        m.insert(7, "Ulm");
        m.insert(8, "Marverni");
        m.insert(9, "Sauromatia");
        m.insert(10, "T'ien Ch'i");
        m.insert(11, "Machaka");
        m.insert(12, "Mictlan");
        m.insert(13, "Abysia");
        m.insert(14, "Caelum");
        m.insert(15, "C'tis");
        m.insert(16, "Pangaea");
        m.insert(17, "Agartha");
        m.insert(18, "Tir na n'Og");
        m.insert(19, "Fomoria");
        m.insert(20, "Vanheim");
        m.insert(21, "Helheim");
        m.insert(22, "Niefelheim");
        m.insert(24, "Rus");
        m.insert(25, "Kailasa");
        m.insert(26, "Lanka");
        m.insert(27, "Yomi");
        m.insert(28, "Hinnom");
        m.insert(29, "Ur");
        m.insert(30, "Berytos");
        m.insert(31, "Xibalba");
        m.insert(32, "Mekone");
        m.insert(33, "Ubar");
        m.insert(36, "Atlantis");
        m.insert(37, "R'lyeh");
        m.insert(38, "Pelagia");
        m.insert(39, "Oceania");
        m.insert(40, "Therodos");
        m
    };

    pub static ref MA_NATIONS_BY_ID: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(43, "Arcoscephale");
        m.insert(44, "Ermor");
        m.insert(45, "Sceleria");
        m.insert(46, "Pythium");
        m.insert(47, "Man");
        m.insert(48, "Eriu");
        m.insert(49, "Ulm");
        m.insert(50, "Marignon");
        m.insert(51, "Mictlan");
        m.insert(52, "T'ien Ch'i");
        m.insert(53, "Machaka");
        m.insert(54, "Agartha");
        m.insert(55, "Abysia");
        m.insert(56, "Caelum");
        m.insert(57, "C'tis");
        m.insert(58, "Pangaea");
        m.insert(59, "Asphodel");
        m.insert(60, "Vanheim");
        m.insert(61, "Jotunheim");
        m.insert(62, "Vanarus");
        m.insert(63, "Bandar Log");
        m.insert(64, "Shinuyama");
        m.insert(65, "Ashdod");
        m.insert(66, "Uruk");
        m.insert(67, "Nazca");
        m.insert(68, "Xibalba");
        m.insert(69, "Phlegra"); // nice
        m.insert(70, "Phaeacia");
        m.insert(71, "Ind");
        m.insert(72, "Na'ba");
        m.insert(73, "Atlantis");
        m.insert(74, "R'lyeh");
        m.insert(75, "Pelagia");
        m.insert(76, "Oceania");
        m.insert(77, "Ys");
        m
    };

    pub static ref LA_NATIONS_BY_ID: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(80, "Arcoscephale");
        m.insert(81, "Pythium");
        m.insert(82, "Lemuria");
        m.insert(83, "Man");
        m.insert(84, "Ulm");
        m.insert(85, "Marignon");
        m.insert(86, "Mictlan");
        m.insert(87, "T'ien Ch'i");
        m.insert(89, "Jomon");
        m.insert(90, "Agartha");
        m.insert(91, "Abysia");
        m.insert(92, "Caelum");
        m.insert(93, "C'tis");
        m.insert(94, "Pangaea");
        m.insert(95, "Midgard");
        m.insert(96, "Utgard");
        m.insert(97, "Bogarus");
        m.insert(98, "Patala");
        m.insert(99, "Gath");
        m.insert(100, "Ragha");
        m.insert(101, "Xibalba");
        m.insert(102, "Phlegra");
        m.insert(103, "Vaettiheim");
        m.insert(106, "Atlantis");
        m.insert(107, "R'lyeh");
        m.insert(108, "Erytheia");
        m
    };

    pub static ref DOM_6_EA_BY_NAME: HashMap<&'static str, (&'static str, u32)> = maplit::hashmap!{
        "arcoscephale golden era" => ("Arcoscephale, Golden Era", 5),
        "mekone brazen giants" => ("Mekone, Brazen Giants", 6),
        "pangaea age of revelry" => ("Pangaea, Age of Revelry", 7),
        "ermor new faith" => ("Ermor, New Faith", 8),
        "sauromatia amazon queens" => ("Sauromatia, Amazon Queens", 9),
        "fomoria the cursed ones" => ("Fomoria, The Cursed Ones", 10),
        "tir na nog land of the ever young" => ("Tir na n'Og, Land of the Ever Young", 11),
        "marverni time of druids" => ("Marverni, Time of Druids", 12),
        "ulm enigma of steel" => ("Ulm, Enigma of Steel", 13),
        "pyrene kingdom of the bekrydes" => ("Pyrène, Kingdom of the Bekrydes", 14),
        "agartha pale ones" => ("Agartha, Pale Ones", 15),
        "abysia children of flame" => ("Abysia, Children of Flame", 16),
        "hinnom sons of the fallen" => ("Hinnom, Sons of the Fallen", 17),
        "ubar kingdom of the unseen" => ("Ubar, Kingdom of the Unseen", 18),
        "ur the first city" => ("Ur, The First City", 19),
        "kailasa rise of the ape kings" => ("Kailasa, Rise of the Ape Kings", 20),
        "lanka land of demons" => ("Lanka, Land of Demons", 21),
        "tien chi spring and autumn" => ("T'ien Ch'i, Spring and Autumn", 22),
        "yomi oni kings" => ("Yomi, Oni Kings", 23),
        "caelum eagle kings" => ("Caelum, Eagle Kings", 24),
        "mictlan reign of blood" => ("Mictlan, Reign of Blood", 25),
        "xibalba vigil of the sun" => ("Xibalba, Vigil of the Sun", 26),
        "ctis lizard kings" => ("C'tis, Lizard Kings", 27),
        "machaka lion kings" => ("Machaka, Lion Kings", 28),
        "berytos the phoenix empire" => ("Berytos, The Phoenix Empire", 29),
        "vanheim age of vanir" => ("Vanheim, Age of Vanir", 30),
        "helheim dusk and death" => ("Helheim, Dusk and Death", 31),
        "rus sons of heaven" => ("Rus, Sons of Heaven", 32),
        "niefelheim sons of winter" => ("Niefelheim, Sons of Winter", 33),
        "muspelheim sons of fire" => ("Muspelheim, Sons of Fire", 34),
        "pelagia pearl kings" => ("Pelagia, Pearl Kings", 40),
        "oceania coming of the capricorns" => ("Oceania, Coming of the Capricorns", 41),
        "therodos telkhine spectre" => ("Therodos, Telkhine Spectre", 42),
        "atlantis emergence of the deep ones" => ("Atlantis, Emergence of the Deep Ones", 43),
        "rlyeh time of aboleths" => ("R'lyeh, Time of Aboleths", 44),
    };
    pub static ref DOM_6_EA_BY_ID: HashMap<u32, &'static str> = {
        let mut hm = HashMap::new();
        for &(k,v) in DOM_6_EA_BY_NAME.values() {
            hm.insert(v,k);
        }
        hm
    };

    pub static ref DOM_6_MA_BY_NAME: HashMap<&'static str, (&'static str, u32)> = maplit::hashmap!{
        "arcoscephale the old kingdom" => ("Arcoscephale, The Old Kingdom", 50),
        "phlegra deformed giants" => ("Phlegra, Deformed Giants", 51),
        "pangaea age of bronze" => ("Pangaea, Age of Bronze", 52),
        "asphodel carrion woods" => ("Asphodel, Carrion Woods", 53),
        "ermor ashen empire" => ("Ermor, Ashen Empire", 54),
        "sceleria the reformed empire" => ("Sceleria, The Reformed Empire", 55),
        "pythium emerald empire" => ("Pythium, Emerald Empire", 56),
        "man tower of avalon" => ("Man, Tower of Avalon", 57),
        "eriu last of the tuatha" => ("Eriu, Last of the Tuatha", 58),
        "agartha golem cult" => ("Agartha, Golem Cult", 59),
        "ulm forges of ulm" => ("Ulm, Forges of Ulm", 60),
        "marignon fiery justice" => ("Marignon, Fiery Justice", 61),
        "pyrene time of the akelarre" => ("Pyrène, Time of the Akelarre", 62),
        "abysia blood and fire" => ("Abysia, Blood and Fire", 63),
        "ashdod reign of the anakim" => ("Ashdod, Reign of the Anakim", 64),
        "naba queens of the desert" => ("Na'Ba, Queens of the Desert", 65),
        "uruk city states" => ("Uruk, City States", 66),
        "ind magnificent kingdom of exalted virtue" => ("Ind, Magnificent Kingdom of Exalted Virtue", 67),
        "bandar log land of the apes" => ("Bandar Log, Land of the Apes", 68),
        "tien chi imperial bureaucracy" => ("T'ien Ch'i, Imperial Bureaucracy", 69),
        "shinuyama land of the bakemono" => ("Shinuyama, Land of the Bakemono", 70),
        "caelum reign of the seraphim" => ("Caelum, Reign of the Seraphim", 71),
        "nazca kingdom of the sun" => ("Nazca, Kingdom of the Sun", 72),
        "mictlan reign of the lawgiver" => ("Mictlan, Reign of the Lawgiver", 73),
        "xibalba flooded caves" => ("Xibalba, Flooded Caves", 74),
        "ctis miasma" => ("C'tis, Miasma", 75),
        "machaka reign of sorcerors" => ("Machaka, Reign of Sorcerors", 76),
        "phaeacia isle of the dark ships" => ("Phaeacia, Isle of the Dark Ships", 77),
        "vanheim arrival of man" => ("Vanheim, Arrival of Man", 78),
        "vanarus land of the chuds" => ("Vanarus, Land of the Chuds", 79),
        "jotunheim iron woods" => ("Jotunheim, Iron Woods", 80),
        "nidavangr bear wolf and crow" => ("Nidavangr, Bear, Wolf and Crow", 81),
        "ys morgen queens" => ("Ys, Morgen Queens", 85),
        "pelagia triton kings" => ("Pelagia, Triton Kings", 86),
        "oceania mermidons" => ("Oceania, Mermidons", 87),
        "atlantis kings of the deep" => ("Atlantis, Kings of the Deep", 88),
        "rlyeh fallen star" => ("R'lyeh, Fallen Star", 89),
    };

    pub static ref DOM_6_MA_BY_ID: HashMap<u32, &'static str> = {
        let mut hm = HashMap::new();
        for &(k,v) in DOM_6_MA_BY_NAME.values() {
            hm.insert(v,k);
        }
        hm
    };

    pub static ref DOM_6_LA_BY_NAME: HashMap<&'static str, (&'static str, u32)> = maplit::hashmap!{
        "arcoscephale sibylline guidance" => ("Arcoscephale, Sibylline Guidance", 95),
        "phlegra sleeping giants" => ("Phlegra, Sleeping Giants", 96),
        "pangaea new era" => ("Pangaea, New Era", 97),
        "pythium serpent cult" => ("Pythium, Serpent Cult", 98),
        "lemuria soul gates" => ("Lemuria, Soul Gates", 99),
        "man towers of chelms" => ("Man, Towers of Chelms", 100),
        "ulm black forest" => ("Ulm, Black Forest", 101),
        "agartha ktonian dead" => ("Agartha, Ktonian Dead", 102),
        "marignon conquerors of the sea" => ("Marignon, Conquerors of the Sea", 103),
        "abysia blood of humans" => ("Abysia, Blood of Humans", 104),
        "ragha dual kingdom" => ("Ragha, Dual Kingdom", 105),
        "caelum return of the raptors" => ("Caelum, Return of the Raptors", 106),
        "gath last of the giants" => ("Gath, Last of the Giants", 107),
        "patala reign of the nagas" => ("Patala, Reign of the Nagas", 108),
        "tien chi barbarian kings" => ("T'ien Ch'i, Barbarian Kings", 109),
        "jomon human daimyos" => ("Jomon, Human Daimyos", 110),
        "mictlan blood and rain" => ("Mictlan, Blood and Rain", 111),
        "xibalba return of the zotz" => ("Xibalba, Return of the Zotz", 112),
        "ctis desert tombs" => ("C'tis, Desert Tombs", 113),
        "midgard age of men" => ("Midgård, Age of Men", 115),
        "bogarus age of heroes" => ("Bogarus, Age of Heroes", 116),
        "utgrd well of urd" => ("Utgård, Well of Urd", 117),
        "vaettiheim wolf kin jarldom" => ("Vaettiheim, Wolf Kin Jarldom", 118),
        "feminie sagequeens" => ("Feminie, Sage-Queens", 119),
        "piconye legacy of the prester king" => ("Piconye, Legacy of the Prester King", 120),
        "andramania dog republic" => ("Andramania, Dog Republic", 121),
        "erytheia kingdom of two worlds" => ("Erytheia, Kingdom of Two Worlds", 125),
        "atlantis frozen sea" => ("Atlantis, Frozen Sea", 126),
        "rlyeh dreamlands" => ("R'lyeh, Dreamlands", 127),
    };

    pub static ref DOM_6_LA_BY_ID: HashMap<u32, &'static str> = {
        let mut hm = HashMap::new();
        for &(k,v) in DOM_6_LA_BY_NAME.values() {
            hm.insert(v,k);
        }
        hm
    };

    pub static ref DOM_6_BY_NAME: HashMap<&'static str, StaticNation> = {
        let mut hm = HashMap::new();
        for (&sname, &(name, id)) in DOM_6_EA_BY_NAME.iter() {
            hm.insert(sname, StaticNation{id, name, era: Era::Early});
        }
        for (&sname, &(name, id)) in DOM_6_MA_BY_NAME.iter() {
            hm.insert(sname, StaticNation{id, name, era: Era::Middle});
        }
        for (&sname, &(name, id)) in DOM_6_LA_BY_NAME.iter() {
            hm.insert(sname, StaticNation{id, name, era: Era::Late});
        }
        hm
    };

}

fn cow_drop(cow: Cow<str>, n: usize) -> Cow<str> {
    match cow {
        Cow::Owned(mut string) => string.split_off(n).into(),
        Cow::Borrowed(b_str) => {
            let (_, ret) = b_str.split_at(n);
            ret.into()
        }
    }
}

fn from_id_from_map(id: u32, era: Era, map: &HashMap<u32, &'static str>) -> Option<StaticNation> {
    map.get(&id).map(|nation_name| StaticNation {
        id,
        name: nation_name,
        era,
    })
}

fn find_nation_options_from_map(
    nations_by_id: &HashMap<u32, &'static str>,
    sanitised_prefix: &str,
    era: Era,
) -> Option<Vec<StaticNation>> {
    // It's not like there's massive amounts of nations I guess, so linear is fine
    let vec = nations_by_id
        .iter()
        .filter(|(_, name)| {
            let sanitised_name = sanitise_text(name.to_owned().to_lowercase().into()); // todo: cow utils
            sanitised_name.starts_with(sanitised_prefix)
        })
        .map(|(&id, name)| StaticNation { id, name, era })
        .collect::<Vec<_>>();
    if !vec.is_empty() {
        Some(vec)
    } else {
        None
    }
}

fn find_nation_options_from_map_6(
    nations_by_name: &HashMap<&'static str, (&'static str, u32)>,
    sanitised_prefix: &str,
    era: Era,
) -> Vec<StaticNation> {
    // It's not like there's massive amounts of nations I guess, so linear is fine
    nations_by_name
        .iter()
        .filter(|(sanitised_name, _)| sanitised_name.starts_with(sanitised_prefix))
        .map(|(_, &(name, id))| StaticNation { id, name, era })
        .collect::<Vec<_>>()
}

fn find_nation_options_no_era_6(sanitised_prefix: &str) -> Vec<StaticNation> {
    // It's not like there's massive amounts of nations I guess, so linear is fine
    DOM_6_BY_NAME
        .iter()
        .filter(|&(&sanitised_name, _)| sanitised_name.starts_with(sanitised_prefix))
        .map(|(_, &nation)| nation)
        .collect::<Vec<_>>()
}
