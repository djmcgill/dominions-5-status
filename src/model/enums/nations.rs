use std::collections::HashMap;

type NationEnum = (&'static str, &'static str);
pub fn get_nation_desc(n: usize) -> &'static NationEnum {
    NATIONS_BY_ID.get(&(n as u32)).unwrap_or_else(
        || {
            println!("unknown nation {}", n);
            &("unknown nation", "")
        }
    )
}
// TODO: actually make an enum
lazy_static ! { // TODO: replace with PHF crate?
    pub static ref NATIONS_BY_ID: HashMap<u32, NationEnum> = { 
        let mut m = HashMap::new();
        m.insert(5u32, ("Arcoscephale", "EA"));
        m.insert(6u32, ("Ermor", "EA"));
        m.insert(7u32, ("Ulm", "EA"));
        m.insert(8u32, ("Marverni", "EA"));
        m.insert(9u32, ("Sauromatia", "EA"));
        m.insert(10u32, ("T’ien Ch’i", "EA"));
        m.insert(11u32, ("Machaka", "EA"));
        m.insert(12u32, ("Mictlan", "EA"));
        m.insert(13u32, ("Abysia", "EA"));
        m.insert(14u32, ("Caelum", "EA"));
        m.insert(15u32, ("C’tis", "EA"));
        m.insert(16u32, ("Pangaea", "EA"));
        m.insert(17u32, ("Agartha", "EA"));
        m.insert(18u32, ("Tir na n'Og", "EA"));
        m.insert(19u32, ("Fomoria", "EA"));
        m.insert(20u32, ("Vanheim", "EA"));
        m.insert(21u32, ("Helheim", "EA"));
        m.insert(22u32, ("Niefelheim", "EA"));
        m.insert(24u32, ("Rus", "EA"));
        m.insert(25u32, ("Kailasa", "EA"));
        m.insert(26u32, ("Lanka", "EA"));
        m.insert(27u32, ("Yomi", "EA"));
        m.insert(28u32, ("Hinnom", "EA"));
        m.insert(29u32, ("Ur", "EA"));
        m.insert(30u32, ("Berytos", "EA"));
        m.insert(31u32, ("Xibalba", "EA"));
        m.insert(36u32, ("Atlantis", "EA"));
        m.insert(37u32, ("R’lyeh", "EA"));
        m.insert(38u32, ("Pelagia", "EA"));
        m.insert(39u32, ("Oceania", "EA"));
        m.insert(40u32, ("Therodos", "EA"));
        m.insert(43u32, ("Arcoscephale", "MA"));
        m.insert(44u32, ("Ermor", "MA"));
        m.insert(45u32, ("Sceleria", "MA"));
        m.insert(46u32, ("Pythium", "MA"));
        m.insert(47u32, ("Man", "MA"));
        m.insert(48u32, ("Eriu", "MA"));
        m.insert(49u32, ("Ulm", "MA"));
        m.insert(50u32, ("Marignon", "MA"));
        m.insert(51u32, ("Mictlan", "MA"));
        m.insert(52u32, ("T’ien Ch’i", "MA"));
        m.insert(53u32, ("Machaka", "MA"));
        m.insert(54u32, ("Agartha", "MA"));
        m.insert(55u32, ("Abysia", "MA"));
        m.insert(56u32, ("Caelum", "MA"));
        m.insert(57u32, ("C’tis", "MA"));
        m.insert(58u32, ("Pangaea", "MA"));
        m.insert(59u32, ("Asphodel", "MA"));
        m.insert(60u32, ("Vanheim", "MA"));
        m.insert(61u32, ("Jotunheim", "MA"));
        m.insert(62u32, ("Vanarus", "MA"));
        m.insert(63u32, ("Bandar Log", "MA"));
        m.insert(64u32, ("Shinuyama", "MA"));
        m.insert(65u32, ("Ashdod", "MA"));
        m.insert(66u32, ("Uruk", "MA"));
        m.insert(67u32, ("Nazca", "MA"));
        m.insert(68u32, ("Xibalba", "MA"));
        m.insert(73u32, ("Atlantis", "MA"));
        m.insert(74u32, ("R’lyeh", "MA"));
        m.insert(75u32, ("Pelagia", "MA"));
        m.insert(76u32, ("Oceania", "MA"));
        m.insert(77u32, ("Ys", "MA"));
        m.insert(80u32, ("Arcoscephale", "LA"));
        m.insert(81u32, ("Pythium", "LA"));
        m.insert(82u32, ("Lemur", "LA"));
        m.insert(83u32, ("Man", "LA"));
        m.insert(84u32, ("Ulm", "LA"));
        m.insert(85u32, ("Marignon", "LA"));
        m.insert(86u32, ("Mictlan", "LA"));
        m.insert(87u32, ("T’ien Ch’i", "LA"));
        m.insert(89u32, ("Jomon", "LA"));
        m.insert(90u32, ("Agartha", "LA"));
        m.insert(91u32, ("Abysia", "LA"));
        m.insert(92u32, ("Caelum", "LA"));
        m.insert(93u32, ("C’tis", "LA"));
        m.insert(94u32, ("Pangaea", "LA"));
        m.insert(95u32, ("Midgård", "LA"));
        m.insert(96u32, ("Utgård", "LA"));
        m.insert(97u32, ("Bogarus", "LA"));
        m.insert(98u32, ("Patala", "LA"));
        m.insert(99u32, ("Gath", "LA"));
        m.insert(100u32, ("Ragha", "LA"));
        m.insert(101u32, ("Xibalba", "LA"));
        m.insert(106u32, ("Atlantis", "LA"));
        m.insert(107u32, ("R’lyeh", "LA"));
        m.insert(108u32, ("Erytheia", "LA"));
        m
    };
}
