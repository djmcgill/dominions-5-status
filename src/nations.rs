use std::collections::HashMap;

pub fn get_nation_desc(n: usize) -> &'static str {
    NATIONS_BY_ID.get(&(n as u32)).unwrap_or_else(
        || {
            println!("unknown nation {}", n);
            &"unknown nation"
        }
    )
}
lazy_static ! { // TODO: replace with PHF crate?
    pub static ref NATIONS_BY_ID: HashMap<u32, &'static str> = { 
        let mut m = HashMap::new();
        m.insert(5u32, "Arcoscephale Golden Era");
        m.insert(6u32, "Ermor New Faith");
        m.insert(7u32, "Ulm Enigma of Steel");
        m.insert(8u32, "Marverni Time of Druids");
        m.insert(9u32, "Sauromatia Amazon Queens");
        m.insert(10u32, "T’ien Ch’i Spring and Autumn");
        m.insert(11u32, "Machaka Lion Kings");
        m.insert(12u32, "Mictlan Reign of Blood");
        m.insert(13u32, "Abysia Children of Flame");
        m.insert(14u32, "Caelum Eagle Kings");
        m.insert(15u32, "C’tis Lizard Kings");
        m.insert(16u32, "Pangaea Age of Revelry");
        m.insert(17u32, "Agartha Pale Ones");
        m.insert(18u32, "Tir na n'Og Land of the Ever Young");
        m.insert(19u32, "Fomoria The Cursed Ones");
        m.insert(20u32, "Vanheim Age of Vanir");
        m.insert(21u32, "Helheim Dusk and Death");
        m.insert(22u32, "Niefelheim Sons of Winter");
        m.insert(24u32, "Rus Sons of Heaven");
        m.insert(25u32, "Kailasa Rise of the Ape Kings");
        m.insert(26u32, "Lanka Land of Demons");
        m.insert(27u32, "Yomi Oni Kings");
        m.insert(28u32, "Hinnom Sons of the Fallen");
        m.insert(29u32, "Ur The First City");
        m.insert(30u32, "Berytos Phoenix Empire");
        m.insert(31u32, "Xibalba Vigil of the Sun");
        m.insert(36u32, "Atlantis Emergence of the Deep Ones");
        m.insert(37u32, "R’lyeh Time of Aboleths");
        m.insert(38u32, "Pelagia Pearl Kings");
        m.insert(39u32, "Oceania Coming of the Capricorns");
        m.insert(40u32, "Therodos Telkhine Spectre");
        m.insert(43u32, "Arcoscephale The Old Kingdom");
        m.insert(44u32, "Ermor Ashen Empire");
        m.insert(45u32, "Sceleria Reformed Empire");
        m.insert(46u32, "Pythium Emerald Empire");
        m.insert(47u32, "Man Tower of Avalon");
        m.insert(48u32, "Eriu Last of the Tuatha");
        m.insert(49u32, "Ulm Forges of Ulm");
        m.insert(50u32, "Marignon Fiery Justice");
        m.insert(51u32, "Mictlan Reign of the Lawgiver");
        m.insert(52u32, "T’ien Ch’i Imperial Bureaucracy");
        m.insert(53u32, "Machaka Reign of Sorcerors");
        m.insert(54u32, "Agartha Golem Cult");
        m.insert(55u32, "Abysia Blood and Fire");
        m.insert(56u32, "Caelum Reign of the Seraphim");
        m.insert(57u32, "C’tis Miasma");
        m.insert(58u32, "Pangaea Age of Bronze");
        m.insert(59u32, "Asphodel Carrion Woods");
        m.insert(60u32, "Vanheim Arrival of Man");
        m.insert(61u32, "Jotunheim Iron Woods");
        m.insert(62u32, "Vanarus Land of the Chuds");
        m.insert(63u32, "Bandar Log Land of the Apes");
        m.insert(64u32, "Shinuyama Land of the Bakemono");
        m.insert(65u32, "Ashdod Reign of the Anakim");
        m.insert(66u32, "Uruk City States");
        m.insert(67u32, "Nazca Kingdom of the Sun");
        m.insert(68u32, "Xibalba Flooded Caves");
        m.insert(73u32, "Atlantis Kings of the Deep");
        m.insert(74u32, "R’lyeh Fallen Star");
        m.insert(75u32, "Pelagia Triton Kings");
        m.insert(76u32, "Oceania Mermidons");
        m.insert(77u32, "Ys Morgen Queens");
        m.insert(80u32, "Arcoscephale Sibylline Guidance");
        m.insert(81u32, "Pythium Serpent Cult");
        m.insert(82u32, "Lemur Soul Gate");
        m.insert(83u32, "Man Towers of Chelms");
        m.insert(84u32, "Ulm Black Forest");
        m.insert(85u32, "Marignon Conquerors of the Sea");
        m.insert(86u32, "Mictlan Blood and Rain");
        m.insert(87u32, "T’ien Ch’i Barbarian Kings");
        m.insert(89u32, "Jomon Human Daimyos");
        m.insert(90u32, "Agartha Ktonian Dead");
        m.insert(91u32, "Abysia Blood of Humans");
        m.insert(92u32, "Caelum Return of the Raptors");
        m.insert(93u32, "C’tis Desert Tombs");
        m.insert(94u32, "Pangaea New Era");
        m.insert(95u32, "Midgård Age of Men");
        m.insert(96u32, "Utgård Well of Urd");
        m.insert(97u32, "Bogarus Age of Heroes");
        m.insert(98u32, "Patala Reign of the Nagas");
        m.insert(99u32, "Gath Last of the Giants");
        m.insert(100u32, "Ragha Dual Kingdom");
        m.insert(101u32, "Xibalba Return of the Zotz");
        m.insert(106u32, "Atlantis Frozen Sea");
        m.insert(107u32, "R’lyeh Dreamlands");
        m.insert(108u32, "Erytheia Kingdom of Two Worlds");
        m
    };
}
