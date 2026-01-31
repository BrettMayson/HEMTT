#[derive(Debug, Default, Clone, Copy)]
pub struct Totals {
    total: u32,
    original: u32,
    english: u32,
    czech: u32,
    french: u32,
    spanish: u32,
    italian: u32,
    polish: u32,
    portuguese: u32,
    russian: u32,
    german: u32,
    korean: u32,
    japanese: u32,
    chinese: u32,
    chinesesimp: u32,
    turkish: u32,
    swedish: u32,
    slovak: u32,
    serbocroatian: u32,
    norwegian: u32,
    icelandic: u32,
    hungarian: u32,
    greek: u32,
    finnish: u32,
    dutch: u32,
    ukrainian: u32,
    danish: u32,
}

macro_rules! field {
    ($field:ident) => {
        paste::paste! {
            pub const fn [<inc_ $field>](&mut self) {
                self.$field += 1;
            }

            #[must_use]
            pub const fn $field(&self) -> u32 {
                self.$field
            }
        }
    };
}

impl Totals {
    pub const fn inc(&mut self) {
        self.total += 1;
    }

    #[must_use]
    pub const fn total(&self) -> u32 {
        self.total
    }

    pub const fn merge(&mut self, other: &Self) {
        self.total += other.total;
        self.original += other.original;
        self.english += other.english;
        self.czech += other.czech;
        self.french += other.french;
        self.spanish += other.spanish;
        self.italian += other.italian;
        self.polish += other.polish;
        self.portuguese += other.portuguese;
        self.russian += other.russian;
        self.german += other.german;
        self.korean += other.korean;
        self.japanese += other.japanese;
        self.chinese += other.chinese;
        self.chinesesimp += other.chinesesimp;
        self.turkish += other.turkish;
        self.swedish += other.swedish;
        self.slovak += other.slovak;
        self.serbocroatian += other.serbocroatian;
        self.norwegian += other.norwegian;
        self.icelandic += other.icelandic;
        self.hungarian += other.hungarian;
        self.greek += other.greek;
        self.finnish += other.finnish;
        self.dutch += other.dutch;
        self.ukrainian += other.ukrainian;
        self.danish += other.danish;
    }

    field!(original);
    field!(english);
    field!(czech);
    field!(french);
    field!(spanish);
    field!(italian);
    field!(polish);
    field!(portuguese);
    field!(russian);
    field!(german);
    field!(korean);
    field!(japanese);
    field!(chinese);
    field!(chinesesimp);
    field!(turkish);
    field!(swedish);
    field!(slovak);
    field!(serbocroatian);
    field!(norwegian);
    field!(icelandic);
    field!(hungarian);
    field!(greek);
    field!(finnish);
    field!(dutch);
    field!(ukrainian);
    field!(danish);
}
