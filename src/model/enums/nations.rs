use crate::model::enums::Era;
use lazy_static::lazy_static;
use std::borrow::Cow;
use std::collections::HashMap;

pub struct Nations;
impl Nations {
    pub fn from_id(id: u32) -> Option<StaticNation> {
        from_id_from_map(id, Era::Early, &*EA_NATIONS_BY_ID)
            .or_else(|| from_id_from_map(id, Era::Middle, &*MA_NATIONS_BY_ID))
            .or_else(|| from_id_from_map(id, Era::Late, &*LA_NATIONS_BY_ID))
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
                    &*EA_NATIONS_BY_ID,
                    sanitised_prefix.as_ref(),
                    Era::Early,
                )
            }
            .or_else(|| {
                find_nation_options_from_map(
                    &*MA_NATIONS_BY_ID,
                    sanitised_prefix.as_ref(),
                    Era::Middle,
                )
            })
            .or_else(|| {
                find_nation_options_from_map(
                    &*LA_NATIONS_BY_ID,
                    sanitised_prefix.as_ref(),
                    Era::Late,
                )
            }),
        }
        .unwrap_or(vec![])
    }
}

fn extract_possible_nation_prefix<'a>(
    lowercase_name_prefix: Cow<'a, str>,
) -> (Cow<'a, str>, Option<Era>) {
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

fn sanitise_text<'a>(lowercase_text: Cow<'a, str>) -> Cow<'a, str> {
    lowercase_text
        .to_owned()
        .replace("'", "")
        .replace(", ", "")
        .into()
    // FIXME: it's too late tonight to figure this stuff out
    // lowercase_text.cow_replace("'", "").cow_replace(", ", "")
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
        m.insert(33, "Mekone");
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
}

fn cow_drop<'a>(cow: Cow<'a, str>, n: usize) -> Cow<'a, str> {
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
        .map({ |(&id, name)| StaticNation { id, name, era } })
        .collect::<Vec<_>>();
    if !vec.is_empty() {
        Some(vec)
    } else {
        None
    }
}
