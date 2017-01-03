// Copyright 2016 James Bendig. See the COPYRIGHT file at the top-level
// directory of this distribution.
//
// Licensed under:
//   the MIT license
//     <LICENSE-MIT or https://opensource.org/licenses/MIT>
//   or the Apache License, Version 2.0
//     <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0>,
// at your option. This file may not be copied, modified, or distributed
// except according to those terms.

use chrono::{Datelike,Local,NaiveDate,NaiveTime,TimeZone};
use chrono::datetime::DateTime;
use chrono::offset::utc::UTC;
use chrono::naive::datetime::NaiveDateTime;
use std::any::Any;
use std::marker::PhantomData;
use std::io::Write;
use std::str::FromStr;

use constant::VALUE_END;
use field_type::FieldType;
use message::{Message,SetValueError};
use rule::Rule;

//Generic Field Types (Sorted Alphabetically)

pub struct BoolTrueOrBlankFieldType;

impl FieldType for BoolTrueOrBlankFieldType {
    type Type = bool;

    fn default_value() -> Self::Type {
        false
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        if bytes.len() == 1 {
            *field = match bytes[0] {
                b'Y' => true,
                b'N' => false,
                _ => return Err(SetValueError::WrongFormat),
            };

            return Ok(())
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(field: &Self::Type) -> bool {
        !field
    }

    fn len(_field: &Self::Type) -> usize {
        1
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(if *field { b"Y" } else { b"N" }).unwrap()
    }
}

pub struct CharFieldType;

impl FieldType for CharFieldType {
    type Type = u8;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        if bytes.len() == 1 {
            *field = bytes[0];
            Ok(())
        }
        else {
            Err(SetValueError::WrongFormat)
        }
    }

    fn is_empty(field: &Self::Type) -> bool {
        *field == 0
    }

    fn len(_field: &Self::Type) -> usize {
        1
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(&[*field]).unwrap()
    }
}

//Country names and codes are from https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
//Last updated: 2016-12-27.
#[derive(Clone,PartialEq)]
pub enum Country {
    Andorra,
    UnitedArabEmirates,
    Afghanistan,
    AntiguaAndBarbuda,
    Anguilla,
    Albania,
    Armenia,
    Angola,
    Antartica,
    Argentina,
    AmericanSomoa,
    Austria,
    Australia,
    Aruba,
    AlandIslands,
    Azerbaijan,
    BosniaAndHerzegovina,
    Barbados,
    Bangladesh,
    Belgium,
    BurkinaFaso,
    Bulgaria,
    Bahrain,
    Burundi,
    Benin,
    SaintBarthelemy,
    Bermuda,
    BruneiDarussalam,
    PlurinationalStateOfBolivia,
    BonarieSintEustatiusAndSaba,
    Brazil,
    Bahamas,
    Bhutan,
    BouvetIsland,
    Botswana,
    Belarus,
    Belize,
    Canada,
    Cocos,
    DemocraticRepublicOfTheCongo,
    CentralAfricanRepublic,
    Congo,
    Switzerland,
    CoteDlvoire, //Ivory Coast
    CookIslands,
    Chile,
    Cameroon,
    China,
    Colombia,
    CostaRica,
    Cuba,
    CaboVerde,
    Cracao,
    ChristmasIsland,
    Cyprus,
    Czechia,
    Germany,
    Djibouti,
    Denmark,
    Dominica,
    DominicanRepublic,
    Algeria,
    Ecuador,
    Estonia,
    Egypt,
    WesternSahara,
    Eritrea,
    Spain,
    Ethiopia,
    Finland,
    Fiji,
    FalklandIslands,
    FederatedStatesOfMicronesia,
    FaroeIslands,
    France,
    Gabon,
    UnitedKingdomOfGreatBritainAndNorthernIreland,
    Grenada,
    Georgia,
    FrenchGuiana,
    Guernsey,
    Ghana,
    Gibraltar,
    Greenland,
    Gambia,
    Guinea,
    Guadeloupe,
    EqatorialGuinea,
    Greece,
    SouthGeorgiaAndTheSouthSandwichIslands,
    Guatemala,
    Guam,
    GuineaBissau,
    Guyana,
    HongKong,
    HeardIslandAndMcDonaldIslands,
    Honduras,
    Croatia,
    Haiti,
    Hungary,
    Indonesia,
    Ireland,
    Israel,
    IsleOfMan,
    India,
    BritishIndianOceanTerritory,
    Iraq,
    IslamicRepublicOfIran,
    Iceland,
    Italy,
    Jersey,
    Jamaica,
    Jordan,
    Japan,
    Kenya,
    Kyrgyzstan,
    Cambodia,
    Kiribati,
    Comoros,
    SaintKittsAndNevis,
    DemocraticPeoplesRepublicOfKorea,
    RepublicOfKorea,
    Kuwait,
    CaymanIslands,
    Kazakhstan,
    LaoPeoplesDemocraticRepublic,
    Lebanon,
    SaintLucia,
    Liechtenstein,
    SriLanka,
    Liberia,
    Lesotho,
    Lithuania,
    Luxembourg,
    Latvia,
    Libya,
    Morocco,
    Monaco,
    RepublicOfMoldova,
    Montenegro,
    SaintMartin,
    Madagascar,
    MarshallIslands,
    TheFormerYugoslavRepublicOfMacedonia,
    Mali,
    Myanmar,
    Mongolia,
    Macao,
    NorthernMarianaIslands,
    Martinique,
    Mauritania,
    Montserrat,
    Malta,
    Mauritius,
    Maldives,
    Malawi,
    Mexico,
    Malaysia,
    Mozambique,
    Namibia,
    NewCaledonia,
    Niger,
    NorfolkIsland,
    Nigeria,
    Nicaragua,
    Netherlands,
    Norway,
    Nepal,
    Nauru,
    Niue,
    NewZealand,
    Oman,
    Panama,
    Peru,
    FrenchPolynesia,
    PapuaNewGuinea,
    Philippines,
    Pakistan,
    Poland,
    SaintPierreAndMiquelon,
    Pitcairn,
    PuertoRico,
    StateOfPalestine,
    Portugal,
    Palau,
    Paraguay,
    Qatar,
    Reunion,
    Romania,
    Serbia,
    RussianFederation,
    Rwanda,
    SaudiArabia,
    SolomonIslands,
    Seychelles,
    Sudan,
    Sweden,
    Singapore,
    AscensionAndTristanDaCunhaSaintHelena,
    Slovenia,
    SvalbardAndJanMayen,
    Slovakia,
    SierraLeone,
    SanMarino,
    Senegal,
    Somalia,
    Suriname,
    SouthSudan,
    SaoTomeAndPrincipe,
    ElSavador,
    SintMaarten,
    SyrianArabRepublic,
    Swaziland,
    TurksAndCaicosIslands,
    Chad,
    FrenchSouthernTerritories,
    Togo,
    Thailand,
    Tajikistan,
    Tokelau,
    TimorLeste,
    Turkmenistan,
    Tunisia,
    Tonga,
    Turkey,
    TrinidadAndTobago,
    Tuvalu,
    ProvinceOfChinaTaiwan,
    UnitedRepublicOfTanzania,
    Ukraine,
    Uganda,
    UnitedStatesMinorOutlyingIslands,
    UnitedStatesOfAmerica,
    Uruguay,
    Uzbekistan,
    HolySee,
    SaintVincentAndTheGrenadines,
    BolivarianRepublicOfVenezuela,
    BritishVirginIslands,
    USVirginIslands,
    VietNam,
    Vanuatu,
    WallisAndFutuna,
    Samoa,
    Yemen,
    Mayotte,
    SouthAfrica,
    Zambia,
    Zimbabwe,
}

define_enum_field_type!(NOT_REQUIRED, Country, CountryFieldType {
    Country::Andorra => b"AD",
    Country::UnitedArabEmirates => b"AE",
    Country::Afghanistan => b"AF",
    Country::AntiguaAndBarbuda => b"AG",
    Country::Anguilla => b"AI",
    Country::Albania => b"AL",
    Country::Armenia => b"AM",
    Country::Angola => b"AO",
    Country::Antartica => b"AQ",
    Country::Argentina => b"AR",
    Country::AmericanSomoa => b"AS",
    Country::Austria => b"AT",
    Country::Australia => b"AU",
    Country::Aruba => b"AW",
    Country::AlandIslands => b"AX",
    Country::Azerbaijan => b"AZ",
    Country::BosniaAndHerzegovina => b"BA",
    Country::Barbados => b"BB",
    Country::Bangladesh => b"BD",
    Country::Belgium => b"BE",
    Country::BurkinaFaso => b"BF",
    Country::Bulgaria => b"BG",
    Country::Bahrain => b"BH",
    Country::Burundi => b"BI",
    Country::Benin => b"BJ",
    Country::SaintBarthelemy => b"BL",
    Country::Bermuda => b"BM",
    Country::BruneiDarussalam => b"BN",
    Country::PlurinationalStateOfBolivia => b"BO",
    Country::BonarieSintEustatiusAndSaba => b"BQ",
    Country::Brazil => b"BR",
    Country::Bahamas => b"BS",
    Country::Bhutan => b"BT",
    Country::BouvetIsland => b"BV",
    Country::Botswana => b"BW",
    Country::Belarus => b"BY",
    Country::Belize => b"BZ",
    Country::Canada => b"CA",
    Country::Cocos => b"CC",
    Country::DemocraticRepublicOfTheCongo => b"CD",
    Country::CentralAfricanRepublic => b"CF",
    Country::Congo => b"CG",
    Country::Switzerland => b"CH",
    Country::CoteDlvoire => b"CI",
    Country::CookIslands => b"CK",
    Country::Chile => b"CL",
    Country::Cameroon => b"CM",
    Country::China => b"CN",
    Country::Colombia => b"CO",
    Country::CostaRica => b"CR",
    Country::Cuba => b"CU",
    Country::CaboVerde => b"CV",
    Country::Cracao => b"CW",
    Country::ChristmasIsland => b"CX",
    Country::Cyprus => b"CY",
    Country::Czechia => b"CZ",
    Country::Germany => b"DE",
    Country::Djibouti => b"DJ",
    Country::Denmark => b"DK",
    Country::Dominica => b"DM",
    Country::DominicanRepublic => b"DO",
    Country::Algeria => b"DZ",
    Country::Ecuador => b"EC",
    Country::Estonia => b"EE",
    Country::Egypt => b"EG",
    Country::WesternSahara => b"EH",
    Country::Eritrea => b"ER",
    Country::Spain => b"ES",
    Country::Ethiopia => b"ET",
    Country::Finland => b"FI",
    Country::Fiji => b"FJ",
    Country::FalklandIslands => b"FK",
    Country::FederatedStatesOfMicronesia => b"FM",
    Country::FaroeIslands => b"FO",
    Country::France => b"FR",
    Country::Gabon => b"GA",
    Country::UnitedKingdomOfGreatBritainAndNorthernIreland => b"GB",
    Country::Grenada => b"GD",
    Country::Georgia => b"GE",
    Country::FrenchGuiana => b"GF",
    Country::Guernsey => b"GG",
    Country::Ghana => b"GH",
    Country::Gibraltar => b"GI",
    Country::Greenland => b"GL",
    Country::Gambia => b"GM",
    Country::Guinea => b"GN",
    Country::Guadeloupe => b"GP",
    Country::EqatorialGuinea => b"GQ",
    Country::Greece => b"GR",
    Country::SouthGeorgiaAndTheSouthSandwichIslands => b"GS",
    Country::Guatemala => b"GT",
    Country::Guam => b"GU",
    Country::GuineaBissau => b"GW",
    Country::Guyana => b"GY",
    Country::HongKong => b"HK",
    Country::HeardIslandAndMcDonaldIslands => b"HM",
    Country::Honduras => b"HN",
    Country::Croatia => b"HR",
    Country::Haiti => b"HT",
    Country::Hungary => b"HU",
    Country::Indonesia => b"ID",
    Country::Ireland => b"IE",
    Country::Israel => b"IL",
    Country::IsleOfMan => b"IM",
    Country::India => b"IN",
    Country::BritishIndianOceanTerritory => b"IO",
    Country::Iraq => b"IQ",
    Country::IslamicRepublicOfIran => b"IR",
    Country::Iceland => b"IS",
    Country::Italy => b"IT",
    Country::Jersey => b"JE",
    Country::Jamaica => b"JM",
    Country::Jordan => b"JO",
    Country::Japan => b"JP",
    Country::Kenya => b"KE",
    Country::Kyrgyzstan => b"KG",
    Country::Cambodia => b"KH",
    Country::Kiribati => b"KI",
    Country::Comoros => b"KM",
    Country::SaintKittsAndNevis => b"KN",
    Country::DemocraticPeoplesRepublicOfKorea => b"KP",
    Country::RepublicOfKorea => b"KR",
    Country::Kuwait => b"KW",
    Country::CaymanIslands => b"KY",
    Country::Kazakhstan => b"KZ",
    Country::LaoPeoplesDemocraticRepublic => b"LA",
    Country::Lebanon => b"LB",
    Country::SaintLucia => b"LC",
    Country::Liechtenstein => b"LI",
    Country::SriLanka => b"LK",
    Country::Liberia => b"LR",
    Country::Lesotho => b"LS",
    Country::Lithuania => b"LT",
    Country::Luxembourg => b"LU",
    Country::Latvia => b"LV",
    Country::Libya => b"LY",
    Country::Morocco => b"MA",
    Country::Monaco => b"MC",
    Country::RepublicOfMoldova => b"MD",
    Country::Montenegro => b"ME",
    Country::SaintMartin => b"MF",
    Country::Madagascar => b"MG",
    Country::MarshallIslands => b"MH",
    Country::TheFormerYugoslavRepublicOfMacedonia => b"MK",
    Country::Mali => b"ML",
    Country::Myanmar => b"MM",
    Country::Mongolia => b"MN",
    Country::Macao => b"MO",
    Country::NorthernMarianaIslands => b"MP",
    Country::Martinique => b"MQ",
    Country::Mauritania => b"MR",
    Country::Montserrat => b"MS",
    Country::Malta => b"MT",
    Country::Mauritius => b"MU",
    Country::Maldives => b"MV",
    Country::Malawi => b"MW",
    Country::Mexico => b"MX",
    Country::Malaysia => b"MY",
    Country::Mozambique => b"MZ",
    Country::Namibia => b"NA",
    Country::NewCaledonia => b"NC",
    Country::Niger => b"NE",
    Country::NorfolkIsland => b"NF",
    Country::Nigeria => b"NG",
    Country::Nicaragua => b"NI",
    Country::Netherlands => b"NL",
    Country::Norway => b"NO",
    Country::Nepal => b"NP",
    Country::Nauru => b"NR",
    Country::Niue => b"NU",
    Country::NewZealand => b"NZ",
    Country::Oman => b"OM",
    Country::Panama => b"PA",
    Country::Peru => b"PE",
    Country::FrenchPolynesia => b"PF",
    Country::PapuaNewGuinea => b"PG",
    Country::Philippines => b"PH",
    Country::Pakistan => b"PK",
    Country::Poland => b"PL",
    Country::SaintPierreAndMiquelon => b"PM",
    Country::Pitcairn => b"PN",
    Country::PuertoRico => b"PR",
    Country::StateOfPalestine => b"PS",
    Country::Portugal => b"PT",
    Country::Palau => b"PW",
    Country::Paraguay => b"PY",
    Country::Qatar => b"QA",
    Country::Reunion => b"RE",
    Country::Romania => b"RO",
    Country::Serbia => b"RS",
    Country::RussianFederation => b"RU",
    Country::Rwanda => b"RW",
    Country::SaudiArabia => b"SA",
    Country::SolomonIslands => b"SB",
    Country::Seychelles => b"SC",
    Country::Sudan => b"SD",
    Country::Sweden => b"SE",
    Country::Singapore => b"SG",
    Country::AscensionAndTristanDaCunhaSaintHelena => b"SH",
    Country::Slovenia => b"SI",
    Country::SvalbardAndJanMayen => b"SJ",
    Country::Slovakia => b"SK",
    Country::SierraLeone => b"SL",
    Country::SanMarino => b"SM",
    Country::Senegal => b"SN",
    Country::Somalia => b"SO",
    Country::Suriname => b"SR",
    Country::SouthSudan => b"SS",
    Country::SaoTomeAndPrincipe => b"ST",
    Country::ElSavador => b"SV",
    Country::SintMaarten => b"SX",
    Country::SyrianArabRepublic => b"SY",
    Country::Swaziland => b"SZ",
    Country::TurksAndCaicosIslands => b"TC",
    Country::Chad => b"TD",
    Country::FrenchSouthernTerritories => b"TF",
    Country::Togo => b"TG",
    Country::Thailand => b"TH",
    Country::Tajikistan => b"TJ",
    Country::Tokelau => b"TK",
    Country::TimorLeste => b"TL",
    Country::Turkmenistan => b"TM",
    Country::Tunisia => b"TN",
    Country::Tonga => b"TO",
    Country::Turkey => b"TR",
    Country::TrinidadAndTobago => b"TT",
    Country::Tuvalu => b"TV",
    Country::ProvinceOfChinaTaiwan => b"TW",
    Country::UnitedRepublicOfTanzania => b"TZ",
    Country::Ukraine => b"UA",
    Country::Uganda => b"UG",
    Country::UnitedStatesMinorOutlyingIslands => b"UM",
    Country::UnitedStatesOfAmerica => b"US",
    Country::Uruguay => b"UY",
    Country::Uzbekistan => b"UZ",
    Country::HolySee => b"VA",
    Country::SaintVincentAndTheGrenadines => b"VC",
    Country::BolivarianRepublicOfVenezuela => b"VE",
    Country::BritishVirginIslands => b"VG",
    Country::USVirginIslands => b"VI",
    Country::VietNam => b"VN",
    Country::Vanuatu => b"VU",
    Country::WallisAndFutuna => b"WF",
    Country::Samoa => b"WS",
    Country::Yemen => b"YE",
    Country::Mayotte => b"YT",
    Country::SouthAfrica => b"ZA",
    Country::Zambia => b"ZM",
    Country::Zimbabwe => b"ZW",
} MUST_BE_STRING);

//Currency and codes are from https://en.wikipedia.org/wiki/ISO_4217
//Last updated: 2016-12-27.
#[derive(Clone,PartialEq)]
pub enum Currency {
    UnitedArabEmiratesDirham,
    AfghanAfghani,
    AlbanianLek,
    ArmenianDram,
    NetherlandsAntilleanGuilder,
    AngolanKwanza,
    ArgentinePeso,
    AustralianDollar,
    ArubanFlorin,
    AzerbaijaniManat,
    BosniaAndHerzegovinaConvertibleMark,
    BarbadosDollar,
    BangladeshiTaka,
    BulgarianLev,
    BahrainiDinar,
    BurundianFranc,
    BermudianDollar,
    BruneiDollar,
    Boliviano,
    BolivianMvdol,
    BrazilianReal,
    BahamianDollar,
    BhutaneseNgultrum,
    BotswanaPula,
    NewBelarusianRuble,
    BelarusianRuble,
    BelizeDollar,
    CandianDollar,
    CongoleseFranc,
    WIREuro,
    SwissFranc,
    WIRFranc,
    UnidadDeFomento,
    ChileanPeso,
    ChineseYuan,
    ColombianPeso,
    UnidadDeValorReal,
    CostaRicanColon,
    CubanConvertiblePeso,
    CubanPeso,
    CapeVerdeEscudo,
    CzechKoruna,
    DjiboutianFranc,
    DanishKrone,
    DominicanPeso,
    AlgerianDinar,
    EgyptianPound,
    EritreanNakfa,
    EthiopianBirr,
    Euro,
    FijiDollar,
    FalklandIslandsPound,
    PoundSterling,
    GeorgianIari,
    GhanaianCedi,
    GibraltarPound,
    GambianDalasi,
    GuineanFranc,
    GuatemalanGuetzal,
    GuyaneseDollar,
    HongKongDollar,
    HonduranLempira,
    CroatianKuna,
    HaitianGourde,
    HungarianForint,
    IndonesianRupiah,
    IsraeliNewShekel,
    IndianRupee,
    IraqiDinar,
    IranianRial,
    IcelandicKrona,
    JamaicanDollar,
    JordanianDinar,
    JapaneseYen,
    KenyanShilling,
    KyrgyzstaniSom,
    CambodianRiel,
    ComoroFranc,
    NorthKoreanWon,
    SouthKoreanWon,
    KuwaitiDinar,
    CaymanIslandsDollar,
    KazakhstaniTenge,
    LaoKip,
    LebanesePound,
    SriLankanRupee,
    LiberianDollar,
    LesothoLoti,
    LibyanDinar,
    MorocanDirham,
    MoldovanLeu,
    MalagasyAriary,
    MacedonianDenar,
    MyanmarKyat,
    MongolianTogrog,
    MacanesePataca,
    MauritanianOuguiya,
    MauritianRupee,
    MaldivianRufiyaa,
    MalawianKwacha,
    MexicanPeso,
    MexicanUnidadDeInversion,
    MalaysianRinggit,
    MozambicanMetical,
    NamibianDollar,
    NigerianNaira,
    NicaraguanCordoba,
    NorwegianKrone,
    NepaleseRupee,
    NewZealandDollar,
    OmaniRial,
    PanamanianBalboa,
    PeruvianSol,
    PapuaNewGuineanKina,
    PhilippinePeso,
    PakistaniRupee,
    PolishZloty,
    ParaguayanGuarani,
    QatariRiyal,
    RomanianLeu,
    SerbianDinar,
    RussianRuble,
    RwandanFranc,
    SaudiRiyal,
    SolomonIslandsDollar,
    SeychellesRupee,
    SudanesePound,
    SwedishKronaOrKronor,
    SingaporeDollar,
    SaintHelenaPound,
    SierraLeoneanLeone,
    SomaliShilling,
    SurinameseDollar,
    SouthSudanesePound,
    SaoTomeAndPrincipeDobra,
    SalvadoranColon,
    SyrianPound,
    SwaziLilangeni,
    ThaiBaht,
    TajikistaniSomoni,
    TurkmenistaniManat,
    TunisianDinar,
    TonganPaanga,
    TurkishLira,
    TrinidadAndTobagoDollar,
    NewTaiwanDollar,
    TanzanianShilling,
    UkranianHryvnia,
    UgandanShilling,
    UnitedStatesDollar,
    UnitedStatesDollarNextDay,
    UruguayPesoEnUnidadesIndexadas, //URUIURUI
    UruguayanPeso,
    UzbekistanSom,
    VenezuelanBolivar,
    VietnameseDong,
    VanuatuVatu,
    SamoanTala,
    CFAFrancBEAC,
    Silver,
    Gold,
    EuropeanCompositeUnit,
    EuropeanMonetaryUnit,
    EuropeanUnitOfAccount9,
    EuropeanUnitOfAccount17,
    EastCaribbeanDollar,
    SpecialDrawingRights,
    CFAFrancBCEAO,
    Palladium,
    CFPFranc,
    Platinum,
    SUCRE,
    Test, //Code reserved for testing purposes
    ADBUnitOfAccount,
    NoCurrency,
    YemeniRial,
    SouthAfricanRand,
    ZambianKwacha,
    ZimbabweanDollar,
}

define_enum_field_type!(NOT_REQUIRED, Currency, CurrencyFieldType {
    Currency::UnitedArabEmiratesDirham => b"AED",
    Currency::AfghanAfghani => b"AFN",
    Currency::AlbanianLek => b"ALL",
    Currency::ArmenianDram => b"AMD",
    Currency::NetherlandsAntilleanGuilder => b"ANG",
    Currency::AngolanKwanza => b"AOA",
    Currency::ArgentinePeso => b"ARS",
    Currency::AustralianDollar => b"AUD",
    Currency::ArubanFlorin => b"AWG",
    Currency::AzerbaijaniManat => b"AZN",
    Currency::BosniaAndHerzegovinaConvertibleMark => b"BAM",
    Currency::BarbadosDollar => b"BBD",
    Currency::BangladeshiTaka => b"BDT",
    Currency::BulgarianLev => b"BGN",
    Currency::BahrainiDinar => b"BHD",
    Currency::BurundianFranc => b"BIF",
    Currency::BermudianDollar => b"BMD",
    Currency::BruneiDollar => b"BND",
    Currency::Boliviano => b"BOB",
    Currency::BolivianMvdol => b"BOV",
    Currency::BrazilianReal => b"BRL",
    Currency::BahamianDollar => b"BSD",
    Currency::BhutaneseNgultrum => b"BTN",
    Currency::BotswanaPula => b"BWP",
    Currency::NewBelarusianRuble => b"BYN",
    Currency::BelarusianRuble => b"BYR",
    Currency::BelizeDollar => b"BZD",
    Currency::CandianDollar => b"CAD",
    Currency::CongoleseFranc => b"CDF",
    Currency::WIREuro => b"CHE",
    Currency::SwissFranc => b"CHF",
    Currency::WIRFranc => b"CHW",
    Currency::UnidadDeFomento => b"CLF",
    Currency::ChileanPeso => b"CLP",
    Currency::ChineseYuan => b"CNY",
    Currency::ColombianPeso => b"COP",
    Currency::UnidadDeValorReal => b"COU",
    Currency::CostaRicanColon => b"CRC",
    Currency::CubanConvertiblePeso => b"CUC",
    Currency::CubanPeso => b"CUP",
    Currency::CapeVerdeEscudo => b"CVE",
    Currency::CzechKoruna => b"CZK",
    Currency::DjiboutianFranc => b"DJF",
    Currency::DanishKrone => b"DKK",
    Currency::DominicanPeso => b"DOP",
    Currency::AlgerianDinar => b"DZD",
    Currency::EgyptianPound => b"EGP",
    Currency::EritreanNakfa => b"ERN",
    Currency::EthiopianBirr => b"ETB",
    Currency::Euro => b"EUR",
    Currency::FijiDollar => b"FJD",
    Currency::FalklandIslandsPound => b"FKP",
    Currency::PoundSterling => b"GBP",
    Currency::GeorgianIari => b"GEL",
    Currency::GhanaianCedi => b"GHS",
    Currency::GibraltarPound => b"GIP",
    Currency::GambianDalasi => b"GMD",
    Currency::GuineanFranc => b"GNF",
    Currency::GuatemalanGuetzal => b"GTQ",
    Currency::GuyaneseDollar => b"GYD",
    Currency::HongKongDollar => b"HKD",
    Currency::HonduranLempira => b"HNL",
    Currency::CroatianKuna => b"HRK",
    Currency::HaitianGourde => b"HTG",
    Currency::HungarianForint => b"HUF",
    Currency::IndonesianRupiah => b"IDR",
    Currency::IsraeliNewShekel => b"ILS",
    Currency::IndianRupee => b"INR",
    Currency::IraqiDinar => b"IQD",
    Currency::IranianRial => b"IRR",
    Currency::IcelandicKrona => b"ISK",
    Currency::JamaicanDollar => b"JMD",
    Currency::JordanianDinar => b"JOD",
    Currency::JapaneseYen => b"JPY",
    Currency::KenyanShilling => b"KES",
    Currency::KyrgyzstaniSom => b"KGS",
    Currency::CambodianRiel => b"KHR",
    Currency::ComoroFranc => b"KMF",
    Currency::NorthKoreanWon => b"KPW",
    Currency::SouthKoreanWon => b"KRW",
    Currency::KuwaitiDinar => b"KWD",
    Currency::CaymanIslandsDollar => b"KYD",
    Currency::KazakhstaniTenge => b"KYZT",
    Currency::LaoKip => b"LAK",
    Currency::LebanesePound => b"LBP",
    Currency::SriLankanRupee => b"LKR",
    Currency::LiberianDollar => b"LRD",
    Currency::LesothoLoti => b"LSL",
    Currency::LibyanDinar => b"LYD",
    Currency::MorocanDirham => b"MAD",
    Currency::MoldovanLeu => b"MDL",
    Currency::MalagasyAriary => b"MGA",
    Currency::MacedonianDenar => b"MKD",
    Currency::MyanmarKyat => b"MMK",
    Currency::MongolianTogrog => b"MNT",
    Currency::MacanesePataca => b"MOP",
    Currency::MauritanianOuguiya => b"MRO",
    Currency::MauritianRupee => b"MUR",
    Currency::MaldivianRufiyaa => b"MVR",
    Currency::MalawianKwacha => b"MWK",
    Currency::MexicanPeso => b"MXN",
    Currency::MexicanUnidadDeInversion => b"MXV",
    Currency::MalaysianRinggit => b"MYR",
    Currency::MozambicanMetical => b"MZN",
    Currency::NamibianDollar => b"NAD",
    Currency::NigerianNaira => b"NGN",
    Currency::NicaraguanCordoba => b"NIO",
    Currency::NorwegianKrone => b"NOK",
    Currency::NepaleseRupee => b"NPR",
    Currency::NewZealandDollar => b"NZD",
    Currency::OmaniRial => b"OMR",
    Currency::PanamanianBalboa => b"PAB",
    Currency::PeruvianSol => b"PEN",
    Currency::PapuaNewGuineanKina => b"PGK",
    Currency::PhilippinePeso => b"PHP",
    Currency::PakistaniRupee => b"PKR",
    Currency::PolishZloty => b"PLN",
    Currency::ParaguayanGuarani => b"PYG",
    Currency::QatariRiyal => b"QAR",
    Currency::RomanianLeu => b"RON",
    Currency::SerbianDinar => b"RSD",
    Currency::RussianRuble => b"RUB",
    Currency::RwandanFranc => b"RWF",
    Currency::SaudiRiyal => b"SAR",
    Currency::SolomonIslandsDollar => b"SBD",
    Currency::SeychellesRupee => b"SCR",
    Currency::SudanesePound => b"SDG",
    Currency::SwedishKronaOrKronor => b"SEK",
    Currency::SingaporeDollar => b"SGD",
    Currency::SaintHelenaPound => b"SHP",
    Currency::SierraLeoneanLeone => b"SLL",
    Currency::SomaliShilling => b"SOS",
    Currency::SurinameseDollar => b"SRD",
    Currency::SouthSudanesePound => b"SSP",
    Currency::SaoTomeAndPrincipeDobra => b"STD",
    Currency::SalvadoranColon => b"SVC",
    Currency::SyrianPound => b"SYP",
    Currency::SwaziLilangeni => b"SZL",
    Currency::ThaiBaht => b"THB",
    Currency::TajikistaniSomoni => b"TJS",
    Currency::TurkmenistaniManat => b"TMT",
    Currency::TunisianDinar => b"TND",
    Currency::TonganPaanga => b"TOP",
    Currency::TurkishLira => b"TRY",
    Currency::TrinidadAndTobagoDollar => b"TTD",
    Currency::NewTaiwanDollar => b"TWD",
    Currency::TanzanianShilling => b"TZS",
    Currency::UkranianHryvnia => b"UAH",
    Currency::UgandanShilling => b"UGX",
    Currency::UnitedStatesDollar => b"USD",
    Currency::UnitedStatesDollarNextDay => b"USN",
    Currency::UruguayPesoEnUnidadesIndexadas => b"UYI",
    Currency::UruguayanPeso => b"UYU",
    Currency::UzbekistanSom => b"UZS",
    Currency::VenezuelanBolivar => b"VEF",
    Currency::VietnameseDong => b"VND",
    Currency::VanuatuVatu => b"VUV",
    Currency::SamoanTala => b"WST",
    Currency::CFAFrancBEAC => b"XAF",
    Currency::Silver => b"XAG",
    Currency::Gold => b"XAU",
    Currency::EuropeanCompositeUnit => b"XBA",
    Currency::EuropeanMonetaryUnit => b"XBB",
    Currency::EuropeanUnitOfAccount9 => b"XBC",
    Currency::EuropeanUnitOfAccount17 => b"XBD",
    Currency::EastCaribbeanDollar => b"XCD",
    Currency::SpecialDrawingRights => b"XDR",
    Currency::CFAFrancBCEAO => b"XOF",
    Currency::Palladium => b"XPD",
    Currency::CFPFranc => b"XPF",
    Currency::Platinum => b"XPT",
    Currency::SUCRE => b"XSU",
    Currency::Test => b"XTS",
    Currency::ADBUnitOfAccount => b"XUA",
    Currency::NoCurrency => b"XXX",
    Currency::YemeniRial => b"YER",
    Currency::SouthAfricanRand => b"ZAR",
    Currency::ZambianKwacha => b"ZMW",
    Currency::ZimbabweanDollar => b"ZWL",
} MUST_BE_STRING);

pub struct DataFieldType;

impl FieldType for DataFieldType {
    type Type = Vec<u8>;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        field.resize(bytes.len(),0);
        field.copy_from_slice(bytes);
        Ok(())
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(field).unwrap()
    }
}

pub struct DayOfMonthFieldType;

impl FieldType for DayOfMonthFieldType {
    type Type = Option<u8>;

    fn default_value() -> Self::Type {
        None
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        let value_string = String::from_utf8_lossy(bytes).into_owned();
        if let Ok(new_value) = u8::from_str(&value_string) {
            if new_value < 1 || new_value > 31 {
                return Err(SetValueError::OutOfRange);
            }

            *field = Some(new_value);

            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_none()
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        if let Some(value) = *field {
            let value_string = value.to_string();
            return buf.write(value_string.as_bytes()).unwrap()
        }

        0
    }
}

pub struct IntFieldType;

impl FieldType for IntFieldType {
    //The spec just says an integer but does not specify a minimum or maximum value.
    //TODO: Investigate if any field will ever need BigInt-style support instead.
    type Type = i64;

    fn default_value() -> Self::Type {
        0
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        let value_string = String::from_utf8_lossy(bytes).into_owned();
        if let Ok(new_value) = Self::Type::from_str(&value_string) {
            *field = new_value;

            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(_field: &Self::Type) -> bool {
        //Always required. Use OptionIntFieldType instead if field is optional.
        false
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let value_string = field.to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

pub struct LocalMktDateFieldType;

impl LocalMktDateFieldType {
    pub fn new_now() -> <LocalMktDateFieldType as FieldType>::Type {
        Local::now().naive_local().date()
    }

    pub fn new_empty() -> <LocalMktDateFieldType as FieldType>::Type {
        //Create a new time stamp that can be considered empty.
        NaiveDate::from_ymd(-1,1,1)
    }
}

impl FieldType for LocalMktDateFieldType {
    type Type = NaiveDate;

    fn default_value() -> Self::Type {
        LocalMktDateFieldType::new_empty()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        //TODO: Share the format string in a constant.
        let value_string = String::from_utf8_lossy(bytes).into_owned();
        if let Ok(new_date) = Self::Type::parse_from_str(&value_string,"%Y%m%d") {
            *field = new_date;

            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.year() < 0
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        assert!(!Self::is_empty(&field)); //Was required field not set?

        let value_string = field.format("%Y%m%d").to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

pub struct NoneFieldType;

impl FieldType for NoneFieldType {
    type Type = PhantomData<()>;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn is_empty(_field: &Self::Type) -> bool {
        true
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(_field: &Self::Type,_buf: &mut Vec<u8>) -> usize {
        0
    }
}

#[derive(Clone,PartialEq)]
pub enum MonthYearRemainder {
    Day(u8),
    Week(u8),
}

#[derive(Clone,PartialEq)]
pub struct MonthYear {
    year: i16,
    month: u8,
    remainder: Option<MonthYearRemainder>,
}

impl MonthYear {
    pub fn new(bytes: &[u8]) -> Option<MonthYear> {
        if bytes.len() < 6 || bytes.len() > 8 {
            return None;
        }

        let value_string = String::from_utf8_lossy(bytes).into_owned();

        let year = if let Ok(year) = i16::from_str(&value_string[0..4]) { year } else { return None };
        if year < 0 || year > 9999 {
            return None;
        }

        let month = if let Ok(month) = u8::from_str(&value_string[4..5]) { month } else { return None };
        if month < 1 || month > 12 {
            return None;
        }

        //Optional remainder portion can be a day of month (1-31) or week number(1-5) with 'w'
        //prefix.
        let remainder = if bytes.len() > 6 {
            Some(if bytes[6] == b'w' {
                MonthYearRemainder::Week(match u8::from_str(&value_string[7..]) {
                    Ok(week) if week >= 1 && week <= 5 => week,
                    _ => return None,
                })
            }
            else {
                MonthYearRemainder::Day(match u8::from_str(&value_string[6..7]) {
                    Ok(day) if day >= 1 && day <= 31 => day,
                    _ => return None,
                })
            })
        }
        else {
            None
        };

        Some(MonthYear {
            year: year,
            month: month,
            remainder: remainder,
        })
    }

    pub fn new_now() -> MonthYear {
        let datetime = UTC::now();

        MonthYear {
            year: datetime.year() as i16,
            month: datetime.month() as u8,
            remainder: None,
        }
    }

    pub fn new_now_day() -> MonthYear {
        let datetime = UTC::now();

        MonthYear {
            year: datetime.year() as i16,
            month: datetime.month() as u8,
            remainder: Some(MonthYearRemainder::Day(datetime.day() as u8)),
        }
    }

    pub fn new_now_with_week(week: u8) -> MonthYear {
        let datetime = UTC::now();

        MonthYear {
            year: datetime.year() as i16,
            month: datetime.month() as u8,
            remainder: Some(MonthYearRemainder::Week(week)),
        }
    }

    pub fn new_empty() -> MonthYear {
        MonthYear {
            year: -1,
            month: 1,
            remainder: None,
        }

    }
}

pub struct MonthYearFieldType;

impl FieldType for MonthYearFieldType {
    type Type = MonthYear;

    fn default_value() -> Self::Type {
        Self::Type::new_empty()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        if let Some(value) = Self::Type::new(bytes) {
            *field = value;
            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(field: &Self::Type) -> bool {
        return field.year < 0
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let month_year_string = format!("{:04}{:02}",field.year,field.month);
        let mut result = buf.write(month_year_string.as_bytes()).unwrap();

        if let Some(ref remainder) = field.remainder {
            let remainder_string = match remainder {
                &MonthYearRemainder::Day(day) => day.to_string(),
                &MonthYearRemainder::Week(week) => format!("w{}",week),
            };
            result += buf.write(remainder_string.as_bytes()).unwrap();
        }

        result
    }
}

pub struct SeqNumFieldType;

impl FieldType for SeqNumFieldType {
    //The spec just says a positive integer but does not specify a maximum value. This should allow
    //one number per millisecond for 5.85 * 10^8 years.
    type Type = u64;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        let value_string = String::from_utf8_lossy(bytes).into_owned();
        if let Ok(new_value) = Self::Type::from_str(&value_string) {
            *field = new_value;

            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(field: &Self::Type) -> bool {
        //First sequence number is 1. Fields where SeqNum can be 0 (ie. ResetRequest::EndSeqNo) are
        //marked as required so they will still be included.
        *field == 0
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let value_string = field.to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

pub struct StringFieldType;

impl FieldType for StringFieldType {
    type Type = String;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        *field = String::from_utf8_lossy(bytes).into_owned();
        Ok(())
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        buf.write(field.as_bytes()).unwrap()
    }
}

pub struct RepeatingGroupFieldType<T: Message + PartialEq> {
    message_type: PhantomData<T>,
}

impl<T: Message + Any + Clone + Default + PartialEq> FieldType for RepeatingGroupFieldType<T> {
    type Type = Vec<Box<T>>;

    fn rule() -> Option<Rule> {
        Some(Rule::BeginGroup{ message: Box::new(<T as Default>::default()) })
    }

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_groups(field: &mut Self::Type,groups: &[Box<Message>]) -> bool {
        field.clear();

        for group in groups {
            match group.as_any().downcast_ref::<T>() {
                //TODO: Avoid the clone below.
                Some(casted_group) => field.push(Box::new(casted_group.clone())),
                None => return false,
            }
        }

        true
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let group_count_str = field.len().to_string();
        let mut result = 1;

        result += buf.write(group_count_str.as_bytes()).unwrap();
        buf.push(VALUE_END);

        for group in field {
            result += group.read_body(buf);
        }

        result
    }
}

pub struct UTCTimeOnlyFieldType;

impl UTCTimeOnlyFieldType {
    pub fn new_now() -> <UTCTimeOnlyFieldType as FieldType>::Type {
        let spec = ::time::get_time();

        let hours = spec.sec % (24 * 60 * 60);
        let minutes = spec.sec % (60 * 60);
        let seconds = spec.sec % 60;

        NaiveTime::from_hms(hours as u32,minutes as u32,seconds as u32)
    }
}

impl FieldType for UTCTimeOnlyFieldType {
    type Type = NaiveTime;

    fn default_value() -> Self::Type {
        UTCTimeOnlyFieldType::new_now()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        //TODO: Support making the .sss, indicating milliseconds, optional.
        //TODO: Share the format string in a constant.
        let value_string = String::from_utf8_lossy(bytes).into_owned();
        if let Ok(new_time) = Self::Type::parse_from_str(&value_string,"%T%.3f") {
            *field = new_time;

            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(_field: &Self::Type) -> bool {
        false //Always required.
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        let value_string = field.format("%T%.3f").to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

pub struct UTCTimestampFieldType;

impl UTCTimestampFieldType {
    pub fn new_now() -> <UTCTimestampFieldType as FieldType>::Type {
        let spec = ::time::get_time();

        //Strip nanoseconds so only whole milliseconds remain (with truncation based rounding).
        //This is because UTCTimestamp does not support sub-millisecond precision.
        let mut nsec = spec.nsec as u32;
        nsec -= nsec % 1_000_000;

        let naive = NaiveDateTime::from_timestamp(spec.sec,nsec);
        DateTime::from_utc(naive,UTC)
    }

    pub fn new_empty() -> <UTCTimestampFieldType as FieldType>::Type {
        //Create a new time stamp that can be considered empty. An Option<_> might be preferred
        //but that would make using the timestamp needlessly complicated.
        UTC.ymd(-1,1,1).and_hms(0,0,0)
    }
}

impl FieldType for UTCTimestampFieldType {
    type Type = DateTime<UTC>;

    fn default_value() -> Self::Type {
        UTCTimestampFieldType::new_empty()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        //TODO: Support making the .sss, indicating milliseconds, optional.
        //TODO: Share the format string in a constant.
        let value_string = String::from_utf8_lossy(bytes).into_owned();
        if let Ok(new_timestamp) = field.offset().datetime_from_str(&value_string,"%Y%m%d-%T%.3f") {
            *field = new_timestamp;

            return Ok(());
        }

        Err(SetValueError::WrongFormat)
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.year() < 0
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,buf: &mut Vec<u8>) -> usize {
        assert!(!Self::is_empty(&field)); //Was required field not set?

        let value_string = field.format("%Y%m%d-%T%.3f").to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

