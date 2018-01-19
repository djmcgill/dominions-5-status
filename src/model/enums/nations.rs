use std::collections::HashMap;
use model::enums::Era;

pub struct Nations;
impl Nations {
    pub fn get_nation_desc(n: usize) -> &'static NationEnum {
        NATIONS_BY_ID.get(&(n as u32)).unwrap_or_else(
            || {
                info!("unknown nation {}", n);
                &("unknown nation", Era::Early) // FIXME
            }
        )
    }
}

type NationEnum = (&'static str, Era);

// TODO: actually make an enum
lazy_static ! { // TODO: replace with PHF crate?
    pub static ref NATIONS_BY_ID: HashMap<u32, NationEnum> = { 
        let mut m = HashMap::new();
        m.insert(5u32, ("Arcoscephale", Era::Early));
        m.insert(6u32, ("Ermor", Era::Early));
        m.insert(7u32, ("Ulm", Era::Early));
        m.insert(8u32, ("Marverni", Era::Early));
        m.insert(9u32, ("Sauromatia", Era::Early));
        m.insert(10u32, ("T'ien Ch'i", Era::Early));
        m.insert(11u32, ("Machaka", Era::Early));
        m.insert(12u32, ("Mictlan", Era::Early));
        m.insert(13u32, ("Abysia", Era::Early));
        m.insert(14u32, ("Caelum", Era::Early));
        m.insert(15u32, ("C'tis", Era::Early));
        m.insert(16u32, ("Pangaea", Era::Early));
        m.insert(17u32, ("Agartha", Era::Early));
        m.insert(18u32, ("Tir na n'Og", Era::Early));
        m.insert(19u32, ("Fomoria", Era::Early));
        m.insert(20u32, ("Vanheim", Era::Early));
        m.insert(21u32, ("Helheim", Era::Early));
        m.insert(22u32, ("Niefelheim", Era::Early));
        m.insert(24u32, ("Rus", Era::Early));
        m.insert(25u32, ("Kailasa", Era::Early));
        m.insert(26u32, ("Lanka", Era::Early));
        m.insert(27u32, ("Yomi", Era::Early));
        m.insert(28u32, ("Hinnom", Era::Early));
        m.insert(29u32, ("Ur", Era::Early));
        m.insert(30u32, ("Berytos", Era::Early));
        m.insert(31u32, ("Xibalba", Era::Early));
        m.insert(36u32, ("Atlantis", Era::Early));
        m.insert(37u32, ("R'lyeh", Era::Early));
        m.insert(38u32, ("Pelagia", Era::Early));
        m.insert(39u32, ("Oceania", Era::Early));
        m.insert(40u32, ("Therodos", Era::Early));
        m.insert(43u32, ("Arcoscephale", Era::Middle));
        m.insert(44u32, ("Ermor", Era::Middle));
        m.insert(45u32, ("Sceleria", Era::Middle));
        m.insert(46u32, ("Pythium", Era::Middle));
        m.insert(47u32, ("Man", Era::Middle));
        m.insert(48u32, ("Eriu", Era::Middle));
        m.insert(49u32, ("Ulm", Era::Middle));
        m.insert(50u32, ("Marignon", Era::Middle));
        m.insert(51u32, ("Mictlan", Era::Middle));
        m.insert(52u32, ("T'ien Ch'i", Era::Middle));
        m.insert(53u32, ("Machaka", Era::Middle));
        m.insert(54u32, ("Agartha", Era::Middle));
        m.insert(55u32, ("Abysia", Era::Middle));
        m.insert(56u32, ("Caelum", Era::Middle));
        m.insert(57u32, ("C'tis", Era::Middle));
        m.insert(58u32, ("Pangaea", Era::Middle));
        m.insert(59u32, ("Asphodel", Era::Middle));
        m.insert(60u32, ("Vanheim", Era::Middle));
        m.insert(61u32, ("Jotunheim", Era::Middle));
        m.insert(62u32, ("Vanarus", Era::Middle));
        m.insert(63u32, ("Bandar Log", Era::Middle));
        m.insert(64u32, ("Shinuyama", Era::Middle));
        m.insert(65u32, ("Ashdod", Era::Middle));
        m.insert(66u32, ("Uruk", Era::Middle));
        m.insert(67u32, ("Nazca", Era::Middle));
        m.insert(68u32, ("Xibalba", Era::Middle));
        m.insert(73u32, ("Atlantis", Era::Middle));
        m.insert(74u32, ("R'lyeh", Era::Middle));
        m.insert(75u32, ("Pelagia", Era::Middle));
        m.insert(76u32, ("Oceania", Era::Middle));
        m.insert(77u32, ("Ys", Era::Middle));
        m.insert(80u32, ("Arcoscephale", Era::Late));
        m.insert(81u32, ("Pythium", Era::Late));
        m.insert(82u32, ("Lemur", Era::Late));
        m.insert(83u32, ("Man", Era::Late));
        m.insert(84u32, ("Ulm", Era::Late));
        m.insert(85u32, ("Marignon", Era::Late));
        m.insert(86u32, ("Mictlan", Era::Late));
        m.insert(87u32, ("T'ien Ch'i", Era::Late));
        m.insert(89u32, ("Jomon", Era::Late));
        m.insert(90u32, ("Agartha", Era::Late));
        m.insert(91u32, ("Abysia", Era::Late));
        m.insert(92u32, ("Caelum", Era::Late));
        m.insert(93u32, ("C'tis", Era::Late));
        m.insert(94u32, ("Pangaea", Era::Late));
        m.insert(95u32, ("Midgård", Era::Late));
        m.insert(96u32, ("Utgård", Era::Late));
        m.insert(97u32, ("Bogarus", Era::Late));
        m.insert(98u32, ("Patala", Era::Late));
        m.insert(99u32, ("Gath", Era::Late));
        m.insert(100u32, ("Ragha", Era::Late));
        m.insert(101u32, ("Xibalba", Era::Late));
        m.insert(106u32, ("Atlantis", Era::Late));
        m.insert(107u32, ("R'lyeh", Era::Late));
        m.insert(108u32, ("Erytheia", Era::Late));
        m
    };
}
