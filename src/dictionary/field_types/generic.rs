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

use chrono::{Datelike,Local,NaiveDate,NaiveTime,Timelike};
use chrono::datetime::DateTime;
use chrono::offset::utc::UTC;
use chrono::naive::datetime::NaiveDateTime;
use std::any::Any;
use std::marker::PhantomData;
use std::io::Write;
use std::str::FromStr;

use constant::VALUE_END;
use field_type::FieldType;
use fix_version::FIXVersion;
use message::{Message,MessageBuildable,SetValueError};
use message_version::MessageVersion;
use rule::Rule;

//Helper function(s)

fn slice_to_int<T: FromStr>(bytes: &[u8]) -> Result<T,SetValueError> {
    //Safe version.
    /*let string = String::from_utf8_lossy(bytes);
    T::from_str(string.as_ref()).map_err(|_| SetValueError::WrongFormat)*/

    //Unsafe version. (Slightly faster and should be okay considering what from_str is
    //doing. Famous last words?)
    use std::str;
    let string = unsafe { str::from_utf8_unchecked(bytes) };
    T::from_str(string).map_err(|_| SetValueError::WrongFormat)
}

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

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
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

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
        buf.write(&[*field]).unwrap()
    }
}

//Country names and codes are from https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
//Last updated: 2016-12-27.
define_enum_field_type!(
    FIELD Country {
        Andorra => b"AD",
        UnitedArabEmirates => b"AE",
        Afghanistan => b"AF",
        AntiguaAndBarbuda => b"AG",
        Anguilla => b"AI",
        Albania => b"AL",
        Armenia => b"AM",
        Angola => b"AO",
        Antartica => b"AQ",
        Argentina => b"AR",
        AmericanSomoa => b"AS",
        Austria => b"AT",
        Australia => b"AU",
        Aruba => b"AW",
        AlandIslands => b"AX",
        Azerbaijan => b"AZ",
        BosniaAndHerzegovina => b"BA",
        Barbados => b"BB",
        Bangladesh => b"BD",
        Belgium => b"BE",
        BurkinaFaso => b"BF",
        Bulgaria => b"BG",
        Bahrain => b"BH",
        Burundi => b"BI",
        Benin => b"BJ",
        SaintBarthelemy => b"BL",
        Bermuda => b"BM",
        BruneiDarussalam => b"BN",
        PlurinationalStateOfBolivia => b"BO",
        BonarieSintEustatiusAndSaba => b"BQ",
        Brazil => b"BR",
        Bahamas => b"BS",
        Bhutan => b"BT",
        BouvetIsland => b"BV",
        Botswana => b"BW",
        Belarus => b"BY",
        Belize => b"BZ",
        Canada => b"CA",
        Cocos => b"CC",
        DemocraticRepublicOfTheCongo => b"CD",
        CentralAfricanRepublic => b"CF",
        Congo => b"CG",
        Switzerland => b"CH",
        CoteDlvoire => b"CI", //Ivory Coast
        CookIslands => b"CK",
        Chile => b"CL",
        Cameroon => b"CM",
        China => b"CN",
        Colombia => b"CO",
        CostaRica => b"CR",
        Cuba => b"CU",
        CaboVerde => b"CV",
        Cracao => b"CW",
        ChristmasIsland => b"CX",
        Cyprus => b"CY",
        Czechia => b"CZ",
        Germany => b"DE",
        Djibouti => b"DJ",
        Denmark => b"DK",
        Dominica => b"DM",
        DominicanRepublic => b"DO",
        Algeria => b"DZ",
        Ecuador => b"EC",
        Estonia => b"EE",
        Egypt => b"EG",
        WesternSahara => b"EH",
        Eritrea => b"ER",
        Spain => b"ES",
        Ethiopia => b"ET",
        Finland => b"FI",
        Fiji => b"FJ",
        FalklandIslands => b"FK",
        FederatedStatesOfMicronesia => b"FM",
        FaroeIslands => b"FO",
        France => b"FR",
        Gabon => b"GA",
        UnitedKingdomOfGreatBritainAndNorthernIreland => b"GB",
        Grenada => b"GD",
        Georgia => b"GE",
        FrenchGuiana => b"GF",
        Guernsey => b"GG",
        Ghana => b"GH",
        Gibraltar => b"GI",
        Greenland => b"GL",
        Gambia => b"GM",
        Guinea => b"GN",
        Guadeloupe => b"GP",
        EqatorialGuinea => b"GQ",
        Greece => b"GR",
        SouthGeorgiaAndTheSouthSandwichIslands => b"GS",
        Guatemala => b"GT",
        Guam => b"GU",
        GuineaBissau => b"GW",
        Guyana => b"GY",
        HongKong => b"HK",
        HeardIslandAndMcDonaldIslands => b"HM",
        Honduras => b"HN",
        Croatia => b"HR",
        Haiti => b"HT",
        Hungary => b"HU",
        Indonesia => b"ID",
        Ireland => b"IE",
        Israel => b"IL",
        IsleOfMan => b"IM",
        India => b"IN",
        BritishIndianOceanTerritory => b"IO",
        Iraq => b"IQ",
        IslamicRepublicOfIran => b"IR",
        Iceland => b"IS",
        Italy => b"IT",
        Jersey => b"JE",
        Jamaica => b"JM",
        Jordan => b"JO",
        Japan => b"JP",
        Kenya => b"KE",
        Kyrgyzstan => b"KG",
        Cambodia => b"KH",
        Kiribati => b"KI",
        Comoros => b"KM",
        SaintKittsAndNevis => b"KN",
        DemocraticPeoplesRepublicOfKorea => b"KP",
        RepublicOfKorea => b"KR",
        Kuwait => b"KW",
        CaymanIslands => b"KY",
        Kazakhstan => b"KZ",
        LaoPeoplesDemocraticRepublic => b"LA",
        Lebanon => b"LB",
        SaintLucia => b"LC",
        Liechtenstein => b"LI",
        SriLanka => b"LK",
        Liberia => b"LR",
        Lesotho => b"LS",
        Lithuania => b"LT",
        Luxembourg => b"LU",
        Latvia => b"LV",
        Libya => b"LY",
        Morocco => b"MA",
        Monaco => b"MC",
        RepublicOfMoldova => b"MD",
        Montenegro => b"ME",
        SaintMartin => b"MF",
        Madagascar => b"MG",
        MarshallIslands => b"MH",
        TheFormerYugoslavRepublicOfMacedonia => b"MK",
        Mali => b"ML",
        Myanmar => b"MM",
        Mongolia => b"MN",
        Macao => b"MO",
        NorthernMarianaIslands => b"MP",
        Martinique => b"MQ",
        Mauritania => b"MR",
        Montserrat => b"MS",
        Malta => b"MT",
        Mauritius => b"MU",
        Maldives => b"MV",
        Malawi => b"MW",
        Mexico => b"MX",
        Malaysia => b"MY",
        Mozambique => b"MZ",
        Namibia => b"NA",
        NewCaledonia => b"NC",
        Niger => b"NE",
        NorfolkIsland => b"NF",
        Nigeria => b"NG",
        Nicaragua => b"NI",
        Netherlands => b"NL",
        Norway => b"NO",
        Nepal => b"NP",
        Nauru => b"NR",
        Niue => b"NU",
        NewZealand => b"NZ",
        Oman => b"OM",
        Panama => b"PA",
        Peru => b"PE",
        FrenchPolynesia => b"PF",
        PapuaNewGuinea => b"PG",
        Philippines => b"PH",
        Pakistan => b"PK",
        Poland => b"PL",
        SaintPierreAndMiquelon => b"PM",
        Pitcairn => b"PN",
        PuertoRico => b"PR",
        StateOfPalestine => b"PS",
        Portugal => b"PT",
        Palau => b"PW",
        Paraguay => b"PY",
        Qatar => b"QA",
        Reunion => b"RE",
        Romania => b"RO",
        Serbia => b"RS",
        RussianFederation => b"RU",
        Rwanda => b"RW",
        SaudiArabia => b"SA",
        SolomonIslands => b"SB",
        Seychelles => b"SC",
        Sudan => b"SD",
        Sweden => b"SE",
        Singapore => b"SG",
        AscensionAndTristanDaCunhaSaintHelena => b"SH",
        Slovenia => b"SI",
        SvalbardAndJanMayen => b"SJ",
        Slovakia => b"SK",
        SierraLeone => b"SL",
        SanMarino => b"SM",
        Senegal => b"SN",
        Somalia => b"SO",
        Suriname => b"SR",
        SouthSudan => b"SS",
        SaoTomeAndPrincipe => b"ST",
        ElSavador => b"SV",
        SintMaarten => b"SX",
        SyrianArabRepublic => b"SY",
        Swaziland => b"SZ",
        TurksAndCaicosIslands => b"TC",
        Chad => b"TD",
        FrenchSouthernTerritories => b"TF",
        Togo => b"TG",
        Thailand => b"TH",
        Tajikistan => b"TJ",
        Tokelau => b"TK",
        TimorLeste => b"TL",
        Turkmenistan => b"TM",
        Tunisia => b"TN",
        Tonga => b"TO",
        Turkey => b"TR",
        TrinidadAndTobago => b"TT",
        Tuvalu => b"TV",
        ProvinceOfChinaTaiwan => b"TW",
        UnitedRepublicOfTanzania => b"TZ",
        Ukraine => b"UA",
        Uganda => b"UG",
        UnitedStatesMinorOutlyingIslands => b"UM",
        UnitedStatesOfAmerica => b"US",
        Uruguay => b"UY",
        Uzbekistan => b"UZ",
        HolySee => b"VA",
        SaintVincentAndTheGrenadines => b"VC",
        BolivarianRepublicOfVenezuela => b"VE",
        BritishVirginIslands => b"VG",
        USVirginIslands => b"VI",
        VietNam => b"VN",
        Vanuatu => b"VU",
        WallisAndFutuna => b"WF",
        Samoa => b"WS",
        Yemen => b"YE",
        Mayotte => b"YT",
        SouthAfrica => b"ZA",
        Zambia => b"ZM",
        Zimbabwe => b"ZW",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_STRING] CountryFieldType
);

//Currency and codes are from https://en.wikipedia.org/wiki/ISO_4217
//Last updated: 2016-12-27.
define_enum_field_type!(
    FIELD Currency {
        UnitedArabEmiratesDirham => b"AED",
        AfghanAfghani => b"AFN",
        AlbanianLek => b"ALL",
        ArmenianDram => b"AMD",
        NetherlandsAntilleanGuilder => b"ANG",
        AngolanKwanza => b"AOA",
        ArgentinePeso => b"ARS",
        AustralianDollar => b"AUD",
        ArubanFlorin => b"AWG",
        AzerbaijaniManat => b"AZN",
        BosniaAndHerzegovinaConvertibleMark => b"BAM",
        BarbadosDollar => b"BBD",
        BangladeshiTaka => b"BDT",
        BulgarianLev => b"BGN",
        BahrainiDinar => b"BHD",
        BurundianFranc => b"BIF",
        BermudianDollar => b"BMD",
        BruneiDollar => b"BND",
        Boliviano => b"BOB",
        BolivianMvdol => b"BOV",
        BrazilianReal => b"BRL",
        BahamianDollar => b"BSD",
        BhutaneseNgultrum => b"BTN",
        BotswanaPula => b"BWP",
        NewBelarusianRuble => b"BYN",
        BelarusianRuble => b"BYR",
        BelizeDollar => b"BZD",
        CandianDollar => b"CAD",
        CongoleseFranc => b"CDF",
        WIREuro => b"CHE",
        SwissFranc => b"CHF",
        WIRFranc => b"CHW",
        UnidadDeFomento => b"CLF",
        ChileanPeso => b"CLP",
        ChineseYuan => b"CNY",
        ColombianPeso => b"COP",
        UnidadDeValorReal => b"COU",
        CostaRicanColon => b"CRC",
        CubanConvertiblePeso => b"CUC",
        CubanPeso => b"CUP",
        CapeVerdeEscudo => b"CVE",
        CzechKoruna => b"CZK",
        DjiboutianFranc => b"DJF",
        DanishKrone => b"DKK",
        DominicanPeso => b"DOP",
        AlgerianDinar => b"DZD",
        EgyptianPound => b"EGP",
        EritreanNakfa => b"ERN",
        EthiopianBirr => b"ETB",
        Euro => b"EUR",
        FijiDollar => b"FJD",
        FalklandIslandsPound => b"FKP",
        PoundSterling => b"GBP",
        GeorgianIari => b"GEL",
        GhanaianCedi => b"GHS",
        GibraltarPound => b"GIP",
        GambianDalasi => b"GMD",
        GuineanFranc => b"GNF",
        GuatemalanGuetzal => b"GTQ",
        GuyaneseDollar => b"GYD",
        HongKongDollar => b"HKD",
        HonduranLempira => b"HNL",
        CroatianKuna => b"HRK",
        HaitianGourde => b"HTG",
        HungarianForint => b"HUF",
        IndonesianRupiah => b"IDR",
        IsraeliNewShekel => b"ILS",
        IndianRupee => b"INR",
        IraqiDinar => b"IQD",
        IranianRial => b"IRR",
        IcelandicKrona => b"ISK",
        JamaicanDollar => b"JMD",
        JordanianDinar => b"JOD",
        JapaneseYen => b"JPY",
        KenyanShilling => b"KES",
        KyrgyzstaniSom => b"KGS",
        CambodianRiel => b"KHR",
        ComoroFranc => b"KMF",
        NorthKoreanWon => b"KPW",
        SouthKoreanWon => b"KRW",
        KuwaitiDinar => b"KWD",
        CaymanIslandsDollar => b"KYD",
        KazakhstaniTenge => b"KYZT",
        LaoKip => b"LAK",
        LebanesePound => b"LBP",
        SriLankanRupee => b"LKR",
        LiberianDollar => b"LRD",
        LesothoLoti => b"LSL",
        LibyanDinar => b"LYD",
        MorocanDirham => b"MAD",
        MoldovanLeu => b"MDL",
        MalagasyAriary => b"MGA",
        MacedonianDenar => b"MKD",
        MyanmarKyat => b"MMK",
        MongolianTogrog => b"MNT",
        MacanesePataca => b"MOP",
        MauritanianOuguiya => b"MRO",
        MauritianRupee => b"MUR",
        MaldivianRufiyaa => b"MVR",
        MalawianKwacha => b"MWK",
        MexicanPeso => b"MXN",
        MexicanUnidadDeInversion => b"MXV",
        MalaysianRinggit => b"MYR",
        MozambicanMetical => b"MZN",
        NamibianDollar => b"NAD",
        NigerianNaira => b"NGN",
        NicaraguanCordoba => b"NIO",
        NorwegianKrone => b"NOK",
        NepaleseRupee => b"NPR",
        NewZealandDollar => b"NZD",
        OmaniRial => b"OMR",
        PanamanianBalboa => b"PAB",
        PeruvianSol => b"PEN",
        PapuaNewGuineanKina => b"PGK",
        PhilippinePeso => b"PHP",
        PakistaniRupee => b"PKR",
        PolishZloty => b"PLN",
        ParaguayanGuarani => b"PYG",
        QatariRiyal => b"QAR",
        RomanianLeu => b"RON",
        SerbianDinar => b"RSD",
        RussianRuble => b"RUB",
        RwandanFranc => b"RWF",
        SaudiRiyal => b"SAR",
        SolomonIslandsDollar => b"SBD",
        SeychellesRupee => b"SCR",
        SudanesePound => b"SDG",
        SwedishKronaOrKronor => b"SEK",
        SingaporeDollar => b"SGD",
        SaintHelenaPound => b"SHP",
        SierraLeoneanLeone => b"SLL",
        SomaliShilling => b"SOS",
        SurinameseDollar => b"SRD",
        SouthSudanesePound => b"SSP",
        SaoTomeAndPrincipeDobra => b"STD",
        SalvadoranColon => b"SVC",
        SyrianPound => b"SYP",
        SwaziLilangeni => b"SZL",
        ThaiBaht => b"THB",
        TajikistaniSomoni => b"TJS",
        TurkmenistaniManat => b"TMT",
        TunisianDinar => b"TND",
        TonganPaanga => b"TOP",
        TurkishLira => b"TRY",
        TrinidadAndTobagoDollar => b"TTD",
        NewTaiwanDollar => b"TWD",
        TanzanianShilling => b"TZS",
        UkranianHryvnia => b"UAH",
        UgandanShilling => b"UGX",
        UnitedStatesDollar => b"USD",
        UnitedStatesDollarNextDay => b"USN",
        UruguayPesoEnUnidadesIndexadas => b"UYI", //URUIURUI
        UruguayanPeso => b"UYU",
        UzbekistanSom => b"UZS",
        VenezuelanBolivar => b"VEF",
        VietnameseDong => b"VND",
        VanuatuVatu => b"VUV",
        SamoanTala => b"WST",
        CFAFrancBEAC => b"XAF",
        Silver => b"XAG",
        Gold => b"XAU",
        EuropeanCompositeUnit => b"XBA",
        EuropeanMonetaryUnit => b"XBB",
        EuropeanUnitOfAccount9 => b"XBC",
        EuropeanUnitOfAccount17 => b"XBD",
        EastCaribbeanDollar => b"XCD",
        SpecialDrawingRights => b"XDR",
        CFAFrancBCEAO => b"XOF",
        Palladium => b"XPD",
        CFPFranc => b"XPF",
        Platinum => b"XPT",
        SUCRE => b"XSU",
        Test => b"XTS", //Code reserved for testing purposes
        ADBUnitOfAccount => b"XUA",
        NoCurrency => b"XXX",
        YemeniRial => b"YER",
        SouthAfricanRand => b"ZAR",
        ZambianKwacha => b"ZMW",
        ZimbabweanDollar => b"ZWL",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_STRING] CurrencyFieldType
);

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

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
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
        let new_value = try!(slice_to_int::<u8>(bytes));
        if new_value < 1 || new_value > 31 {
            return Err(SetValueError::OutOfRange);
        }

        *field = Some(new_value);

        Ok(())
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_none()
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
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
        *field = try!(slice_to_int::<Self::Type>(bytes));

        Ok(())
    }

    fn is_empty(_field: &Self::Type) -> bool {
        //Always required. Use OptionIntFieldType instead if field is optional.
        false
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
        let value_string = field.to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

//LengthFieldType is used identically to SeqNumFieldType.
pub type LengthFieldType = SeqNumFieldType;

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
        if bytes.len() != 8 {
            return Err(SetValueError::WrongFormat);
        }

        let year = try!(slice_to_int::<i32>(&bytes[0..4]));
        let month = try!(slice_to_int::<u32>(&bytes[4..6]));
        let day = try!(slice_to_int::<u32>(&bytes[6..8]));

        *field = NaiveDate::from_ymd(year,month,day);

        Ok(())
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.year() < 0
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
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

    fn read(_field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,_buf: &mut Vec<u8>) -> usize {
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

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
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
        0
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        *field = try!(slice_to_int::<Self::Type>(bytes));

        Ok(())
    }

    fn is_empty(field: &Self::Type) -> bool {
        //First sequence number is 1. Fields where SeqNum can be 0 (ie. ResetRequest::EndSeqNo) are
        //marked as required so they will still be included.
        *field == 0
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
        let value_string = field.to_string();
        buf.write(value_string.as_bytes()).unwrap()
    }
}

pub struct StringFieldType;

impl FieldType for StringFieldType {
    type Type = Vec<u8>;

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        field.clear();
        field.extend_from_slice(bytes);
        Ok(())
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_empty()
    }

    fn len(field: &Self::Type) -> usize {
        field.len()
    }

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
        buf.write(&field[..]).unwrap()
    }
}

pub struct RepeatingGroupFieldType<T: Message + PartialEq> {
    message_type: PhantomData<T>,
}

impl<T: Message + MessageBuildable + Any + Clone + Default + PartialEq + Send + Sized> FieldType for RepeatingGroupFieldType<T> {
    type Type = Vec<Box<T>>;

    fn rule() -> Option<Rule> {
        let message = <T as Default>::default();
        Some(Rule::BeginGroup{ builder_func: <T as MessageBuildable>::builder_func(&message) })
    }

    fn default_value() -> Self::Type {
        Default::default()
    }

    fn set_groups(field: &mut Self::Type,mut groups: Vec<Box<Message>>) -> bool {
        field.clear();

        for group in groups.drain(0..) {
            if group.as_any().is::<T>() {
                let group_ptr = Box::into_raw(group);
                field.push(unsafe {
                    Box::from_raw(group_ptr as *mut T)
                });
            }
            else {
                return false;
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

    fn read(field: &Self::Type,fix_version: FIXVersion,message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
        let group_count_str = field.len().to_string();
        let mut result = 1;

        result += buf.write(group_count_str.as_bytes()).unwrap();
        buf.push(VALUE_END);

        for group in field {
            result += group.read_body(fix_version,message_version,buf);
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
        if bytes.len() < 8 || bytes[2] != b':' || bytes[5] != b':' {
            return Err(SetValueError::WrongFormat);
        }

        let hours = try!(slice_to_int::<u32>(&bytes[0..2]));
        let minutes = try!(slice_to_int::<u32>(&bytes[3..5]));
        let seconds = try!(slice_to_int::<u32>(&bytes[6..8]));
        let milliseconds = if bytes.len() == 8 {
            0
        }
        else if bytes.len() == 12 {
            if bytes[8] != b'.' {
                return Err(SetValueError::WrongFormat);
            }

            try!(slice_to_int::<u32>(&bytes[9..12]))
        }
        else {
            return Err(SetValueError::WrongFormat);
        };

        *field = NaiveTime::from_hms_milli(hours,minutes,seconds,milliseconds);

        Ok(())
    }

    fn is_empty(_field: &Self::Type) -> bool {
        false //Always required.
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
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
        DateTime::<UTC>::from_utc(
            NaiveDate::from_ymd(-1,1,1).and_hms(0,0,0),
            UTC
        )
    }
}

impl FieldType for UTCTimestampFieldType {
    type Type = DateTime<UTC>;

    fn default_value() -> Self::Type {
        UTCTimestampFieldType::new_empty()
    }

    fn set_value(field: &mut Self::Type,bytes: &[u8]) -> Result<(),SetValueError> {
        if bytes.len() < 17 || bytes[8] != b'-' || bytes[11] != b':' || bytes[14] != b':' {
            return Err(SetValueError::WrongFormat);
        }

        let year = try!(slice_to_int::<i32>(&bytes[0..4]));
        let month = try!(slice_to_int::<u32>(&bytes[4..6]));
        let day = try!(slice_to_int::<u32>(&bytes[6..8]));
        let hours = try!(slice_to_int::<u32>(&bytes[9..11]));
        let minutes = try!(slice_to_int::<u32>(&bytes[12..14]));
        let seconds = try!(slice_to_int::<u32>(&bytes[15..17]));
        let milliseconds = if bytes.len() == 17 {
            0
        }
        else if bytes.len() == 21 {
            if bytes[17] != b'.' {
                return Err(SetValueError::WrongFormat);
            }

            try!(slice_to_int::<u32>(&bytes[18..21]))
        }
        else {
            return Err(SetValueError::WrongFormat);
        };

        *field = DateTime::<UTC>::from_utc(
            NaiveDate::from_ymd(year,month,day)
                       .and_hms_milli(hours,minutes,seconds,milliseconds),
            UTC
        );

        Ok(())
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.year() < 0
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(field: &Self::Type,_fix_version: FIXVersion,_message_version: MessageVersion,buf: &mut Vec<u8>) -> usize {
        assert!(!Self::is_empty(&field)); //Was required field not set?

        buf.reserve(21);
        let naive_utc = field.naive_utc();
        write!(buf,
               "{:04}{:02}{:02}-{:02}:{:02}:{:02}.{:03}",
               naive_utc.year(),
               naive_utc.month(),
               naive_utc.day(),
               naive_utc.hour(),
               naive_utc.minute(),
               naive_utc.second(),
               naive_utc.nanosecond() / 1_000_000).unwrap();

        21
    }
}

