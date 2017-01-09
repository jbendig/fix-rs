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

use dictionary::field_types::generic::{BoolTrueOrBlankFieldType,CharFieldType,CountryFieldType,CurrencyFieldType,DataFieldType,DayOfMonthFieldType,IntFieldType,LocalMktDateFieldType,MonthYearFieldType,NoneFieldType,RepeatingGroupFieldType,SeqNumFieldType,StringFieldType,UTCTimeOnlyFieldType,UTCTimestampFieldType};
use dictionary::field_types::other as other_field_types;
use dictionary::field_types::other::{ApplVerIDFieldType,BusinessRejectReasonFieldType,ComplexEventConditionFieldType,ComplexEventPriceBoundaryMethodFieldType,ComplexEventPriceTimeTypeFieldType,ComplexEventTypeFieldType,ContractMultiplierUnitFieldType,CPProgramFieldType,DefaultApplVerIDFieldType,EmailTypeFieldType,EventTypeFieldType,ExerciseStyleFieldType,FlowScheduleTypeFieldType,HandlInstFieldType,InstrmtAssignmentMethodFieldType,IssuerFieldType,ListMethodFieldType,NotRequiredSecurityIDSourceFieldType,NotRequiredSecurityTypeFieldType as SecurityTypeFieldType,NotRequiredSideFieldType,NotRequiredSymbolSfxFieldType as SymbolSfxFieldType,NotRequiredTimeUnitFieldType as TimeUnitFieldType,OptPayoutTypeFieldType,OrdTypeFieldType,PartyIDSourceFieldType,PartyRoleFieldType,PartySubIDTypeFieldType,PriceQuoteMethodFieldType,ProductFieldType,PutOrCallFieldType,RateSourceFieldType,RateSourceTypeFieldType,RequiredSecurityIDSourceFieldType,RequiredSideFieldType,RequiredStipulationTypeFieldType as StipulationTypeFieldType,RestructuringTypeFieldType,RoutingTypeFieldType,SecurityStatusFieldType,SeniorityFieldType,SessionRejectReasonFieldType,SettlMethodFieldType,SettlTypeFieldType,StrikePriceBoundaryMethodFieldType,StrikePriceDeterminationMethodFieldType,TimeInForceFieldType,UnderlyingCashTypeFieldType,UnderlyingFXRateCalcFieldType,UnderlyingPriceDeterminationMethodFieldType,UnderlyingSettlementTypeFieldType,UnitOfMeasureFieldType,ValuationMethodFieldType};
use fix_version::FIXVersion;
use message::{REQUIRED,NOT_REQUIRED};
use rule::Rule;

//TODO: Create implementations for all of these types.
type PercentageFieldType = StringFieldType;
type PriceFieldType = StringFieldType;
type TZTimeOnlyFieldType = StringFieldType;
type AmtFieldType = StringFieldType;
type QtyFieldType = StringFieldType;
type ExchangeFieldType = StringFieldType; //See ISO 10383 for a complete list: https://www.iso20022.org/10383/iso-10383-market-identifier-codes

define_fields!(
    Account: StringFieldType = b"1",
    BeginSeqNo: SeqNumFieldType = b"7",
    ClOrdID: StringFieldType = b"11",
    Currency: CurrencyFieldType = b"15",
    EndSeqNo: SeqNumFieldType = b"16",
    HandlInst: HandlInstFieldType = b"21",
    SecurityIDSource: NotRequiredSecurityIDSourceFieldType = b"22",
    NoLinesOfText: RepeatingGroupFieldType<LinesOfTextGrp> = b"33",
    MsgSeqNum: SeqNumFieldType = b"34", //TODO: Special field probably might be better off built into the parser.
    NewSeqNo: SeqNumFieldType = b"36",
    OrderID: StringFieldType = b"37",
    OrderQty: StringFieldType = b"38", //Qty
    OrdType: OrdTypeFieldType = b"40",
    OrigTime: UTCTimestampFieldType = b"42",
    PossDupFlag: BoolTrueOrBlankFieldType = b"43",
    Price: StringFieldType = b"44", //Price
    RefSeqNum: SeqNumFieldType = b"45",
    RelatedSym: StringFieldType = b"46", //TODO: Old special field that can be repeated without a repeating group.
    SecurityID: StringFieldType = b"48",
    SenderCompID: StringFieldType = b"49",
    SenderSubID: StringFieldType = b"50",
    SendingTime: UTCTimestampFieldType = b"52",
    SideField: RequiredSideFieldType = b"54",
    Symbol: StringFieldType = b"55",
    TargetCompID: StringFieldType = b"56",
    TargetSubID: StringFieldType = b"57",
    Text: StringFieldType = b"58",
    TimeInForce: TimeInForceFieldType = b"59",
    TransactTime: UTCTimestampFieldType = b"60",
    SettlType: SettlTypeFieldType = b"63",
    SettlDate: LocalMktDateFieldType = b"64",
    SymbolSfx: SymbolSfxFieldType = b"65",
    NoOrders: RepeatingGroupFieldType<Order> = b"73",
    NoAllocs: RepeatingGroupFieldType<Alloc> = b"78",
    AllocAccount: StringFieldType = b"79",
    Signature: DataFieldType = b"89" => Rule::ConfirmPreviousTag{ previous_tag: SignatureLength::tag() },
    SecureDataLen: NoneFieldType = b"90" => Rule::PrepareForBytes{ bytes_tag: SecureData::tag() },
    SecureData: DataFieldType = b"91" => Rule::ConfirmPreviousTag{ previous_tag: SecureDataLen::tag() },
    SignatureLength: NoneFieldType = b"93" => Rule::PrepareForBytes{ bytes_tag: Signature::tag() },
    EmailType: EmailTypeFieldType = b"94",
    RawDataLength: NoneFieldType = b"95" => Rule::PrepareForBytes{ bytes_tag: RawData::tag() },
    RawData: DataFieldType = b"96" => Rule::ConfirmPreviousTag{ previous_tag: RawDataLength::tag() },
    PossResend: StringFieldType = b"97", //Bool
    EncryptMethod: StringFieldType = b"98",
    Issuer: IssuerFieldType = b"106",
    SecurityDesc: StringFieldType = b"107",
    HeartBtInt: IntFieldType = b"108",
    MinQty: StringFieldType = b"110", //Qty
    MaxFloor: StringFieldType = b"111", //Qty
    TestReqID: StringFieldType = b"112",
    OnBehalfOfCompID: StringFieldType = b"115",
    OnBehalfOfSubID: StringFieldType = b"116",
    OrigSendingTime: UTCTimestampFieldType = b"122",
    GapFillFlag: BoolTrueOrBlankFieldType = b"123",
    DeliverToCompID: StringFieldType = b"128",
    DeliverToSubID: StringFieldType = b"129",
    BidSize: StringFieldType = b"134", //Qty
    ResetSeqNumFlag: BoolTrueOrBlankFieldType = b"141",
    SenderLocationID: StringFieldType = b"142",
    TargetLocationID: StringFieldType = b"143",
    OnBehalfOfLocationID: StringFieldType = b"144",
    DeliverToLocationID: StringFieldType = b"145",
    NoRelatedSym: RepeatingGroupFieldType<Instrument> = b"146",
    Subject: StringFieldType = b"147",
    CashOrderQty: StringFieldType = b"152", //Qty
    EmailThreadID: StringFieldType = b"164",
    SecurityType: SecurityTypeFieldType = b"167",
    MaturityMonthYear: MonthYearFieldType = b"200",
    PutOrCall: PutOrCallFieldType = b"201",
    StrikePrice: StringFieldType = b"202", //Price
    MaturityDay: DayOfMonthFieldType = b"205",
    OptAttribute: CharFieldType = b"206",
    SecurityExchange: ExchangeFieldType = b"207",
    XmlDataLen: NoneFieldType = b"212" => Rule::PrepareForBytes{ bytes_tag: XmlData::tag() },
    XmlData: DataFieldType = b"213" => Rule::ConfirmPreviousTag{ previous_tag: XmlDataLen::tag() },
    NoRoutingIDs: RepeatingGroupFieldType<RoutingGrp> = b"215",
    RoutingType: RoutingTypeFieldType = b"216",
    RoutingID: StringFieldType = b"217",
    CouponRate: PercentageFieldType = b"223",
    CouponPaymentDate: LocalMktDateFieldType = b"224", //TODO: Use UTCDate when FIX version < 4.4.
    IssueDate: LocalMktDateFieldType = b"225", //TODO: Use UTCDate when FIX version < 4.4.
    RepurchaseTerm: IntFieldType = b"226",
    RepurchaseRate: PercentageFieldType = b"227",
    Factor: StringFieldType = b"228", //Float
    ContractMultiplier: StringFieldType = b"231", //Float
    RepoCollateralSecurityType: SecurityTypeFieldType = b"239",
    RedemptionDate: LocalMktDateFieldType = b"240",
    UnderlyingCouponPaymentDate: LocalMktDateFieldType = b"241",
    UnderlyingIssueDate: LocalMktDateFieldType = b"242",
    UnderlyingRepoCollateralSecurityType: SecurityTypeFieldType = b"243",
    UnderlyingRepurchaseTerm: IntFieldType = b"244",
    UnderlyingRepurchaseRate: PercentageFieldType = b"245",
    UnderlyingFactor: StringFieldType = b"246", //Float
    UnderlyingRedemptionDate: LocalMktDateFieldType = b"247",
    LegCouponPaymentDate: LocalMktDateFieldType = b"248",
    LegIssueDate: LocalMktDateFieldType = b"249",
    LegRepoCollateralSecurityType: SecurityTypeFieldType = b"250",
    LegRepurchaseTerm: IntFieldType = b"251",
    LegRepurchaseRate: PercentageFieldType = b"252",
    LegFactor: StringFieldType = b"253", //Float
    LegRedemptionDate: LocalMktDateFieldType = b"254",
    CreditRating: StringFieldType = b"255",
    UnderlyingCreditRating: StringFieldType = b"256",
    LegCreditRating: StringFieldType = b"257",
    UnderlyingSecurityIDSource: NotRequiredSecurityIDSourceFieldType = b"305",
    UnderlyingIssuer: IssuerFieldType = b"306",
    UnderlyingSecurityDesc: StringFieldType = b"307",
    UnderlyingSecurityExchange: ExchangeFieldType = b"308",
    UnderlyingSecurityID: StringFieldType = b"309",
    UnderlyingSecurityType: SecurityTypeFieldType = b"310",
    UnderlyingSymbol: StringFieldType = b"311",
    UnderlyingSymbolSfx: SymbolSfxFieldType = b"312",
    UnderlyingMaturityMonthYear: MonthYearFieldType = b"313",
    UnderlyingPutOrCall: PutOrCallFieldType = b"315",
    UnderlyingStrikePrice: StringFieldType = b"316", //Price
    UnderlyingOptAttribute: CharFieldType = b"317",
    UnderlyingCurrency: CurrencyFieldType = b"318",
    MessageEncoding: StringFieldType = b"347",
    EncodedIssuerLen: NoneFieldType = b"348" => Rule::PrepareForBytes{ bytes_tag: EncodedIssuer::tag() },
    EncodedIssuer: DataFieldType = b"349" => Rule::ConfirmPreviousTag{ previous_tag: EncodedIssuerLen::tag() },
    EncodedSecurityDescLen: NoneFieldType = b"350" => Rule::PrepareForBytes{ bytes_tag: EncodedSecurityDesc::tag() },
    EncodedSecurityDesc: DataFieldType = b"351" => Rule::ConfirmPreviousTag{ previous_tag: EncodedSecurityDescLen::tag() },
    EncodedTextLen: NoneFieldType = b"354" => Rule::PrepareForBytes{ bytes_tag: EncodedText::tag() },
    EncodedText: DataFieldType = b"355" => Rule::ConfirmPreviousTag{ previous_tag: EncodedTextLen::tag() },
    EncodedSubjectLen: NoneFieldType = b"356" => Rule::PrepareForBytes{ bytes_tag: EncodedSubject::tag() },
    EncodedSubject: DataFieldType = b"357" => Rule::ConfirmPreviousTag{ previous_tag: EncodedSubjectLen::tag() },
    EncodedUnderlyingIssuerLen: NoneFieldType = b"362" => Rule::PrepareForBytes{ bytes_tag: EncodedUnderlyingIssuer::tag() },
    EncodedUnderlyingIssuer: DataFieldType = b"363" => Rule::ConfirmPreviousTag{ previous_tag: EncodedUnderlyingIssuerLen::tag() },
    EncodedUnderlyingSecurityDescLen: NoneFieldType = b"364" => Rule::PrepareForBytes{ bytes_tag: EncodedUnderlyingSecurityDesc::tag() },
    EncodedUnderlyingSecurityDesc: DataFieldType = b"365" => Rule::ConfirmPreviousTag{ previous_tag: EncodedUnderlyingSecurityDescLen::tag() },
    LastMsgSeqNumProcessed: SeqNumFieldType = b"369",
    OnBehalfOfSendingTime: UTCTimestampFieldType = b"370",
    RefTagID: StringFieldType = b"371", //int
    RefMsgType: StringFieldType = b"372",
    SessionRejectReason: SessionRejectReasonFieldType = b"373",
    BusinessRejectRefID: StringFieldType = b"379",
    BusinessRejectReason: BusinessRejectReasonFieldType = b"380",
    MaxMessageSize: StringFieldType = b"383", //Length
    NoMsgTypeGrp: RepeatingGroupFieldType<MsgTypeGrp> = b"384",
    MsgDirection: StringFieldType = b"385", //Char
    UnderlyingCouponRate: PercentageFieldType = b"435",
    UnderlyingContractMultiplier: StringFieldType = b"436", //Float
    NoSecurityAltID: RepeatingGroupFieldType<SecAltIDGrp> = b"454",
    SecurityAltID: StringFieldType = b"455",
    SecurityAltIDSource: RequiredSecurityIDSourceFieldType = b"456",
    NoUnderlyingSecurityAltID: RepeatingGroupFieldType<UndSecAltIDGrp> = b"457",
    UnderlyingSecurityAltID: StringFieldType = b"458",
    UnderlyingSecurityAltIDSource: RequiredSecurityIDSourceFieldType = b"459",
    Product: ProductFieldType = b"460",
    CFICode: StringFieldType = b"461",
    UnderlyingProduct: ProductFieldType = b"462",
    UnderlyingCFICode: StringFieldType = b"463",
    TestMessageIndicator: StringFieldType = b"464", //Bool
    CountryOfIssue: CountryFieldType = b"470",
    StateOrProvinceOfIssue: StringFieldType = b"471",
    LocaleOfIssue: StringFieldType = b"472", //Full code list is available for purchase here: http://www.iata.org/publications/store/Pages/airline-coding-directory.aspx
    MaturityDate: LocalMktDateFieldType = b"541",
    UnderlyingMaturityDate: LocalMktDateFieldType = b"542",
    InstrRegistry: StringFieldType = b"543",
    Username: StringFieldType = b"553",
    Password: StringFieldType = b"554",
    NoLegs: RepeatingGroupFieldType<InstrumentLeg> = b"555",
    LegCurrency: CurrencyFieldType = b"556",
    LegPrice: PriceFieldType = b"566",
    UnderlyingCountryOfIssue: CountryFieldType = b"592",
    UnderlyingStateOrProvinceOfIssue: StringFieldType = b"593",
    UnderlyingLocaleOfIssue: StringFieldType = b"594", //See LocaleOfIssue (472).
    UnderlyingInstrRegistry: StringFieldType = b"595",
    LegCountryOfIssue: CountryFieldType = b"596",
    LegStateOrProvinceOfIssue: StringFieldType = b"597",
    LegLocaleOfIssue: StringFieldType = b"598", //See LocaleOfIssue (472).
    LegInstrRegistry: StringFieldType = b"599",
    LegSymbol: StringFieldType = b"600",
    LegSymbolSfx: SymbolSfxFieldType = b"601",
    LegSecurityID: StringFieldType = b"602",
    LegSecurityIDSource: NotRequiredSecurityIDSourceFieldType = b"603",
    NoLegSecurityAltID: RepeatingGroupFieldType<LegSecAltIDGrp> = b"604",
    LegSecurityAltID: StringFieldType = b"605",
    LegSecurityAltIDSource: RequiredSecurityIDSourceFieldType = b"606",
    LegProduct: ProductFieldType = b"607",
    LegCFICode: StringFieldType = b"608",
    LegSecurityType: SecurityTypeFieldType = b"609",
    LegMaturityMonthYear: MonthYearFieldType = b"610",
    LegMaturityDate: LocalMktDateFieldType = b"611",
    LegStrikePrice: PriceFieldType = b"612",
    LegOptAttribute: CharFieldType = b"613",
    LegContractMultiplier: StringFieldType = b"614", //Float
    LegCouponRate: PercentageFieldType = b"615",
    LegSecurityExchange: ExchangeFieldType = b"616",
    LegIssuer: IssuerFieldType = b"617",
    EncodedLegIssuerLen: NoneFieldType = b"618" => Rule::PrepareForBytes{ bytes_tag: EncodedLegIssuer::tag() },
    EncodedLegIssuer: DataFieldType = b"619" => Rule::ConfirmPreviousTag{ previous_tag: EncodedLegIssuerLen::tag() },
    LegSecurityDesc: StringFieldType = b"620",
    EncodedLegSecurityDescLen: NoneFieldType = b"621" => Rule::PrepareForBytes{ bytes_tag: EncodedLegSecurityDesc::tag() },
    EncodedLegSecurityDesc: DataFieldType = b"622" => Rule::ConfirmPreviousTag{ previous_tag: EncodedLegSecurityDescLen::tag() },
    LegRatioQty: StringFieldType = b"623", //Float
    LegSide: NotRequiredSideFieldType = b"624",
    NoHops: RepeatingGroupFieldType<HopGrp> = b"627",
    HopCompID: StringFieldType = b"628",
    HopSendingTime: UTCTimestampFieldType = b"629",
    HopRefID: SeqNumFieldType = b"630",
    ContractSettlMonth: MonthYearFieldType = b"667",
    Pool: StringFieldType = b"691",
    NoUnderlyings: RepeatingGroupFieldType<UnderlyingInstrument> = b"711",
    LegDatedDate: LocalMktDateFieldType = b"739",
    LegPool: StringFieldType = b"740",
    SecuritySubType: StringFieldType = b"762",
    UnderlyingSecuritySubType: StringFieldType = b"763",
    LegSecuritySubType: StringFieldType = b"764",
    NextExpectedMsgSeqNum: SeqNumFieldType = b"789",
    UnderlyingPx: PriceFieldType = b"810",
    NoEvents: RepeatingGroupFieldType<EvntGrp> = b"864",
    EventType: EventTypeFieldType = b"865",
    EventDate: LocalMktDateFieldType = b"866",
    EventPx: PriceFieldType = b"867",
    EventText: StringFieldType = b"868",
    DatedDate: LocalMktDateFieldType = b"873",
    InterestAccrualDate: LocalMktDateFieldType = b"874",
    CPProgram: CPProgramFieldType = b"875",
    CPRegType: StringFieldType = b"876",
    UnderlyingCPProgram: CPProgramFieldType = b"877",
    UnderlyingCPRegType: StringFieldType = b"878",
    UnderlyingQty: QtyFieldType = b"879",
    UnderlyingDirtyPrice: PriceFieldType = b"882",
    UnderlyingEndPrice: PriceFieldType = b"883",
    UnderlyingStartValue: AmtFieldType = b"884",
    UnderlyingCurrentValue: AmtFieldType = b"885",
    UnderlyingEndValue: AmtFieldType = b"886",
    NoUnderlyingStips: RepeatingGroupFieldType<UnderlyingStipulation> = b"887",
    UnderlyingStipType: StipulationTypeFieldType = b"888",
    UnderlyingStipValue: StringFieldType = b"889", //TODO: Parsable expression.
    NewPassword: StringFieldType = b"925",
    UnderlyingStrikeCurrency: CurrencyFieldType = b"941",
    LegStrikeCurrency: CurrencyFieldType = b"942",
    StrikeCurrency: CurrencyFieldType = b"947",
    LegContractSettlMonth: MonthYearFieldType = b"955",
    LegInterestAccrualDate: LocalMktDateFieldType = b"956",
    SecurityStatus: SecurityStatusFieldType = b"965",
    SettleOnOpenFlag: StringFieldType = b"966",
    StrikeMultiplier: StringFieldType = b"967", //Float
    StrikeValue: StringFieldType = b"968", //Float
    MinPriceIncrement: StringFieldType = b"969", //Float
    PositionLimit: IntFieldType = b"970",
    NTPositionLimit: IntFieldType = b"971",
    UnderlyingAllocationPercent: PercentageFieldType = b"972",
    UnderlyingCashAmount: AmtFieldType = b"973",
    UnderlyingCashType: UnderlyingCashTypeFieldType = b"974",
    UnderlyingSettlementType: UnderlyingSettlementTypeFieldType = b"975",
    UnitOfMeasure: UnitOfMeasureFieldType = b"996",
    TimeUnit: TimeUnitFieldType = b"997",
    UnderlyingUnitOfMeasure: UnitOfMeasureFieldType = b"998",
    LegUnitOfMeasure: UnitOfMeasureFieldType = b"999",
    UnderlyingTimeUnit: TimeUnitFieldType = b"1000",
    LegTimeUnit: TimeUnitFieldType = b"1001",
    LegOptionRatio: StringFieldType = b"1017", //Float
    NoInstrumentParties: RepeatingGroupFieldType<InstrumentParty> = b"1018",
    InstrumentPartyID: StringFieldType = b"1019", //Valid PartyID values are dependent on PartyIDSource and PartyRole.
    UnderlyingDeliveryAmount: AmtFieldType = b"1037",
    UnderlyingCapValue: AmtFieldType = b"1038",
    UnderlyingSettlMethod: StringFieldType = b"1039",
    UnderlyingAdjustedQuantity: StringFieldType = b"1044", //Qty
    UnderlyingFXRate: StringFieldType = b"1045", //Float
    UnderlyingFXRateCalc: UnderlyingFXRateCalcFieldType = b"1046",
    NoUndlyInstrumentParties: RepeatingGroupFieldType<UndlyInstrumentPtysSubGrp> = b"1058",
    InstrmtAssignmentMethod: InstrmtAssignmentMethodFieldType = b"1049",
    InstrumentPartyIDSource: PartyIDSourceFieldType = b"1050",
    InstrumentPartyRole: PartyRoleFieldType = b"1051",
    NoInstrumentPartySubIDs: RepeatingGroupFieldType<InstrumentPtysSubGrp> = b"1052",
    InstrumentPartySubID: StringFieldType = b"1053",
    InstrumentPartySubIDType: PartySubIDTypeFieldType = b"1054",
    NoUndlyInstrumentPartySubIDs: RepeatingGroupFieldType<UndlyInstrumentPtysSubGrp> = b"1062",
    UnderlyingInstrumentPartySubID: StringFieldType = b"1063",
    UnderlyingInstrumentPartySubIDType: PartySubIDTypeFieldType = b"1064",
    MaturityTime: TZTimeOnlyFieldType = b"1079",
    ApplVerID: ApplVerIDFieldType = b"1128" => Rule::RequiresFIXVersion{ fix_version: FIXVersion::FIXT_1_1 },
    CstmApplVerID: StringFieldType = b"1129",
    RefApplVerID: ApplVerIDFieldType = b"1130",
    RefCstmApplVerID: StringFieldType = b"1131",
    DefaultApplVerID: DefaultApplVerIDFieldType = b"1137",
    EventTime: UTCTimestampFieldType = b"1145",
    MinPriceIncrementAmount: AmtFieldType = b"1146",
    UnitOfMeasureQty: StringFieldType = b"1147", //Qty
    SecurityGroup: StringFieldType = b"1151",
    ApplExtID: StringFieldType = b"1156", //int
    SecurityXMLLen: NoneFieldType = b"1184" => Rule::PrepareForBytes{ bytes_tag: SecurityXML::tag() },
    SecurityXML: DataFieldType = b"1185" => Rule::ConfirmPreviousTag{ previous_tag: SecurityXMLLen::tag() },
    SecurityXMLSchema: StringFieldType = b"1186",
    PriceUnitOfMeasure: UnitOfMeasureFieldType = b"1191",
    PriceUnitOfMeasureQty: StringFieldType = b"1192", //Qty
    SettlMethod: SettlMethodFieldType = b"1193",
    ExerciseStyle: ExerciseStyleFieldType = b"1194",
    OptPayoutAmount: AmtFieldType = b"1195",
    PriceQuoteMethod: PriceQuoteMethodFieldType = b"1196",
    ValuationMethod: ValuationMethodFieldType = b"1197",
    ListMethod: ListMethodFieldType = b"1198",
    CapPrice: PriceFieldType = b"1199",
    FloorPrice: PriceFieldType = b"1200",
    LegMaturityTime: TZTimeOnlyFieldType = b"1212",
    UnderlyingMaturityTime: TZTimeOnlyFieldType = b"1213",
    LegUnitOfMeasureQty: StringFieldType = b"1224", //Qty
    ProductComplex: StringFieldType = b"1227",
    FlexibleProductElgibilityIndicator: BoolTrueOrBlankFieldType = b"1242",
    FlexibleIndicator: BoolTrueOrBlankFieldType = b"1244",
    LegPutOrCall: PutOrCallFieldType = b"1358",
    EncryptedPasswordMethod: StringFieldType = b"1400", //int
    EncryptedPasswordLen: NoneFieldType = b"1401" => Rule::PrepareForBytes{ bytes_tag: EncryptedPassword::tag() },
    EncryptedPassword: DataFieldType = b"1402" => Rule::ConfirmPreviousTag{ previous_tag: EncryptedPasswordLen::tag() },
    EncryptedNewPasswordLen: NoneFieldType = b"1403" => Rule::PrepareForBytes{ bytes_tag: EncryptedNewPassword::tag() },
    EncryptedNewPassword: DataFieldType = b"1404" => Rule::ConfirmPreviousTag{ previous_tag: EncryptedNewPasswordLen::tag() },
    RefApplExtID: StringFieldType = b"1406", //int
    DefaultApplExtID: StringFieldType = b"1407", //int
    DefaultCstmApplVerID: StringFieldType = b"1408",
    SessionStatus: StringFieldType = b"1409", //int
    DefaultVerIndicator: StringFieldType = b"1410", //bool
    UnderlyingExerciseStyle: ExerciseStyleFieldType = b"1419",
    LegExerciseStyle: ExerciseStyleFieldType = b"1420",
    LegPriceUnitOfMeasure: UnitOfMeasureFieldType = b"1421",
    LegPriceUnitOfMeasureQty: StringFieldType = b"1422", //Qty
    UnderlyingUnitOfMeasureQty: StringFieldType = b"1423", //Qty
    UnderlyingPriceUnitOfMeasure: UnitOfMeasureFieldType = b"1424",
    UnderlyingPriceUnitOfMeasureQty: StringFieldType = b"1425", //Qty
    ContractMultiplierUnit: ContractMultiplierUnitFieldType = b"1435",
    LegContractMultiplierUnit: ContractMultiplierUnitFieldType = b"1436",
    UnderlyingContractMultiplierUnit: ContractMultiplierUnitFieldType = b"1437",
    FlowScheduleType: FlowScheduleTypeFieldType = b"1439",
    LegFlowScheduleType: FlowScheduleTypeFieldType = b"1440",
    UnderlyingFlowScheduleType: FlowScheduleTypeFieldType = b"1441",
    NoRateSources: RepeatingGroupFieldType<RateSourceGrp> = b"1445",
    RateSource: RateSourceFieldType = b"1446",
    RateSourceType: RateSourceTypeFieldType = b"1447",
    ReferencePage: StringFieldType = b"1448",
    RestructuringType: RestructuringTypeFieldType = b"1449",
    Seniority: SeniorityFieldType = b"1450",
    NotionalPercentageOutstanding: PercentageFieldType = b"1451",
    OriginalNotionalPercentageOutstanding: PercentageFieldType = b"1452",
    UnderlyingRestructuringType: RestructuringTypeFieldType = b"1453",
    UnderlyingSeniority: SeniorityFieldType = b"1454",
    UnderlyingNotionalPercentageOutstanding: PercentageFieldType = b"1455",
    UnderlyingOriginalNotionalPercentageOutstanding: PercentageFieldType = b"1456",
    AttachmentPoint: PercentageFieldType = b"1457",
    DetachmentPoint: PercentageFieldType = b"1458",
    UnderlyingAttachmentPoint: PercentageFieldType = b"1459",
    UnderlyingDetachmentPoint: PercentageFieldType = b"1460",
    StrikePriceDeterminationMethod: StrikePriceDeterminationMethodFieldType = b"1478",
    StrikePriceBoundaryMethod: StrikePriceBoundaryMethodFieldType = b"1479",
    StrikePriceBoundaryPrecision: PercentageFieldType = b"1480",
    UnderlyingPriceDeterminationMethod: UnderlyingPriceDeterminationMethodFieldType = b"1481",
    OptPayoutType: OptPayoutTypeFieldType = b"1482",
    NoComplexEvents: RepeatingGroupFieldType<ComplexEvent> = b"1483",
    ComplexEventType: ComplexEventTypeFieldType = b"1484",
    ComplexOptPayoutAmount: AmtFieldType = b"1485",
    ComplexEventPrice: PriceFieldType = b"1486",
    ComplexEventPriceBoundaryMethod: ComplexEventPriceBoundaryMethodFieldType = b"1487",
    ComplexEventPriceBoundaryPrecision: PercentageFieldType = b"1488",
    ComplexEventPriceTimeType: ComplexEventPriceTimeTypeFieldType = b"1489",
    ComplexEventCondition: ComplexEventConditionFieldType = b"1490",
    NoComplexEventDates: RepeatingGroupFieldType<ComplexEventDate> = b"1491",
    ComplexEventStartDate: UTCTimestampFieldType = b"1492", //TODO: Must always be less than end date.
    ComplexEventEndDate: UTCTimestampFieldType = b"1493", //TODO: Must always be greater than event start date.
    NoComplexEventTimes: RepeatingGroupFieldType<ComplexEventTime> = b"1494",
    ComplexEventStartTime: UTCTimeOnlyFieldType = b"1495", //TODO: Must always be less than end time.
    ComplexEventEndTime: UTCTimeOnlyFieldType = b"1496", //TODO: Must always be greater than start time.
);

//Repeating Groups (Sorted Alphabetically)

define_message!(Alloc {
    REQUIRED, alloc_account: AllocAccount [FIX40..FIX50SP2],
});

define_message!(ComplexEvent {
    REQUIRED, complex_event_type: ComplexEventType [FIX50SP2..],
    NOT_REQUIRED, complex_opt_payout_amount: ComplexOptPayoutAmount [FIX50SP2..],
    NOT_REQUIRED, complex_event_price: ComplexEventPrice [FIX50SP2..],
    NOT_REQUIRED, complex_event_price_boundary_method: ComplexEventPriceBoundaryMethod [FIX50SP2..],
    NOT_REQUIRED, complex_event_price_boundary_precision: ComplexEventPriceBoundaryPrecision [FIX50SP2..],
    NOT_REQUIRED, complex_event_price_time_type: ComplexEventPriceTimeType [FIX50SP2..],
    NOT_REQUIRED, complex_event_condition: ComplexEventCondition [FIX50SP2..], //TODO: Conditionally required only when there is more than one ComplexEvent.
    NOT_REQUIRED, no_complex_event_dates: NoComplexEventDates [FIX50SP2..],
});

define_message!(ComplexEventDate {
    REQUIRED, complex_event_start_date: ComplexEventStartDate [FIX50SP2..],
    REQUIRED, complex_event_end_date: ComplexEventEndDate [FIX50SP2..],
    NOT_REQUIRED, no_complex_event_times: NoComplexEventTimes [FIX50SP2..],
});

define_message!(ComplexEventTime {
    REQUIRED, complex_event_start_time: ComplexEventStartTime [FIX50SP2..],
    REQUIRED, complex_event_end_time: ComplexEventEndTime [FIX50SP2..],
});

define_message!(EvntGrp {
    REQUIRED, event_type: EventType [FIX44..],
    NOT_REQUIRED, event_date: EventDate [FIX44..],
    NOT_REQUIRED, event_time: EventTime [FIX50SP1..],
    NOT_REQUIRED, event_px: EventPx [FIX44..],
    NOT_REQUIRED, event_text: EventText [FIX44..],
});

define_message!(HopGrp {
    REQUIRED, hop_comp_id: HopCompID [FIX43..],
    NOT_REQUIRED, hop_sending_time: HopSendingTime [FIX43..],
    NOT_REQUIRED, hop_ref_id: HopRefID [FIX43..],
});

define_message!(Instrument {
    REQUIRED, related_sym: RelatedSym [FIX42],
    REQUIRED, symbol: Symbol [FIX43..],
    NOT_REQUIRED, symbol_sfx: SymbolSfx [FIX41..],
    NOT_REQUIRED, security_id: SecurityID [FIX41..],
    NOT_REQUIRED, security_id_source: SecurityIDSource [FIX41..] => REQUIRED_WHEN |message: &Instrument,_| { !message.security_id.is_empty() },
    NOT_REQUIRED, no_security_alt_id: NoSecurityAltID [FIX43..],
    NOT_REQUIRED, product: Product [FIX43..],
    NOT_REQUIRED, product_complex: ProductComplex [FIX50SP1..],
    NOT_REQUIRED, security_group: SecurityGroup [FIX50SP1..],
    NOT_REQUIRED, cfi_code: CFICode [FIX43..],
    NOT_REQUIRED, security_type: SecurityType [FIX41..] => REQUIRED_WHEN |message: &Instrument,_| { !message.security_sub_type.is_empty() },
    NOT_REQUIRED, security_sub_type: SecuritySubType [FIX44..],
    NOT_REQUIRED, maturity_month_year: MaturityMonthYear [FIX41..],
    NOT_REQUIRED, maturity_month_day: MaturityDay [FIX41..FIX42],
    NOT_REQUIRED, maturity_date: MaturityDate [FIX43..],
    NOT_REQUIRED, maturity_time: MaturityTime [FIX50..],
    NOT_REQUIRED, settle_on_open_flag: SettleOnOpenFlag [FIX50..],
    NOT_REQUIRED, instrmt_assignment_method: InstrmtAssignmentMethod [FIX50..],
    NOT_REQUIRED, security_status: SecurityStatus [FIX50..],
    NOT_REQUIRED, coupon_payment_date: CouponPaymentDate [FIX43..],
    NOT_REQUIRED, restructuring_type: RestructuringType [FIX50SP2..],
    NOT_REQUIRED, seniority: Seniority [FIX50SP2..],
    NOT_REQUIRED, notional_percentage_outstanding: NotionalPercentageOutstanding [FIX50SP2..],
    NOT_REQUIRED, original_notional_percentage_outstanding: OriginalNotionalPercentageOutstanding [FIX50SP2..],
    NOT_REQUIRED, attachment_point: AttachmentPoint [FIX50SP2..],
    NOT_REQUIRED, detachment_point: DetachmentPoint [FIX50SP2..],
    NOT_REQUIRED, issue_date: IssueDate [FIX43..],
    NOT_REQUIRED, repo_collateral_security_type: RepoCollateralSecurityType [FIX43..],
    NOT_REQUIRED, repurchase_term: RepurchaseTerm [FIX43..],
    NOT_REQUIRED, repurchase_rate: RepurchaseRate [FIX43..],
    NOT_REQUIRED, factor: Factor [FIX43..],
    NOT_REQUIRED, credit_rating: CreditRating [FIX43..],
    NOT_REQUIRED, instr_registry: InstrRegistry [FIX43..],
    NOT_REQUIRED, country_of_issue: CountryOfIssue [FIX43..],
    NOT_REQUIRED, state_or_province_of_issue: StateOrProvinceOfIssue [FIX43..],
    NOT_REQUIRED, locale_of_issue: LocaleOfIssue [FIX43..],
    NOT_REQUIRED, redemption_date: RedemptionDate [FIX43..],
    NOT_REQUIRED, strike_price: StrikePrice [FIX41..],
    NOT_REQUIRED, strike_currency: StrikeCurrency [FIX44..],
    NOT_REQUIRED, strike_multiplier: StrikeMultiplier [FIX50..],
    NOT_REQUIRED, strike_value: StrikeValue [FIX50..],
    NOT_REQUIRED, strike_price_determination_method: StrikePriceDeterminationMethod [FIX50SP2..],
    NOT_REQUIRED, strike_price_boundary_method: StrikePriceBoundaryMethod [FIX50SP2..],
    NOT_REQUIRED, strike_price_boundary_precision: StrikePriceBoundaryPrecision [FIX50SP2..],
    NOT_REQUIRED, underlying_price_determination_method: UnderlyingPriceDeterminationMethod [FIX50SP2..],
    NOT_REQUIRED, opt_attribute: OptAttribute [FIX41..],
    NOT_REQUIRED, contract_multiplier: ContractMultiplier [FIX42..],
    NOT_REQUIRED, contract_multiplier_unit: ContractMultiplierUnit [FIX50SP2..],
    NOT_REQUIRED, flow_schedule_type: FlowScheduleType [FIX50SP2..],
    NOT_REQUIRED, min_price_increment: MinPriceIncrement [FIX50..],
    NOT_REQUIRED, min_price_increment_amount: MinPriceIncrementAmount [FIX50SP1..],
    NOT_REQUIRED, unit_of_measure: UnitOfMeasure [FIX50..],
    NOT_REQUIRED, unit_of_measure_qty: UnitOfMeasureQty [FIX50SP1..],
    NOT_REQUIRED, price_unit_of_measure: PriceUnitOfMeasure [FIX50SP1..],
    NOT_REQUIRED, price_unit_of_measure_qty: PriceUnitOfMeasureQty [FIX50SP1..],
    NOT_REQUIRED, settl_method: SettlMethod [FIX50SP1..],
    NOT_REQUIRED, exercise_style: ExerciseStyle [FIX50SP1..],
    NOT_REQUIRED, opt_payout_type: OptPayoutType [FIX50SP2..],
    NOT_REQUIRED, opt_payout_amount: OptPayoutAmount [FIX50SP1..] => REQUIRED_WHEN |message: &Instrument,_| { if let Some(ref opt_payout_type) = message.opt_payout_type { *opt_payout_type == other_field_types::OptPayoutType::Binary } else { false } },
    NOT_REQUIRED, price_quote_method: PriceQuoteMethod [FIX50SP1..],
    NOT_REQUIRED, valuation_method: ValuationMethod [FIX50SP1..],
    NOT_REQUIRED, list_method: ListMethod [FIX50SP1..],
    NOT_REQUIRED, cap_price: CapPrice [FIX50SP1..],
    NOT_REQUIRED, floor_price: FloorPrice [FIX50SP1..],
    NOT_REQUIRED, put_or_call: PutOrCall [FIX41..],
    NOT_REQUIRED, flexible_indicator: FlexibleIndicator [FIX50SP1..],
    NOT_REQUIRED, flexible_product_eligibility_indicator: FlexibleProductElgibilityIndicator [FIX50SP1..],
    NOT_REQUIRED, time_unit: TimeUnit [FIX50..],
    NOT_REQUIRED, coupon_rate: CouponRate [FIX42..],
    NOT_REQUIRED, security_exchange: SecurityExchange [FIX41..],
    NOT_REQUIRED, position_limit: PositionLimit [FIX50..],
    NOT_REQUIRED, nt_position_limit: NTPositionLimit [FIX50..],
    NOT_REQUIRED, issuer: Issuer [FIX41..],
    NOT_REQUIRED, encoded_issuer_len: EncodedIssuerLen [FIX42..],
    NOT_REQUIRED, encoded_issuer: EncodedIssuer [FIX42..],
    NOT_REQUIRED, security_desc: SecurityDesc [FIX41..],
    NOT_REQUIRED, encoded_security_desc_len: EncodedSecurityDescLen [FIX42..],
    NOT_REQUIRED, encoded_security_desc: EncodedSecurityDesc [FIX42..],
    NOT_REQUIRED, security_xml_len: SecurityXMLLen [FIX50SP1..],
    NOT_REQUIRED, security_xml: SecurityXML [FIX50SP1..],
    NOT_REQUIRED, security_xml_schema: SecurityXMLSchema [FIX50SP1..],
    NOT_REQUIRED, pool: Pool [FIX44..],
    NOT_REQUIRED, contract_settl_month: ContractSettlMonth [FIX44..],
    NOT_REQUIRED, cp_program: CPProgram [FIX44..],
    NOT_REQUIRED, cp_reg_type: CPRegType [FIX44..],
    NOT_REQUIRED, no_events: NoEvents [FIX44..],
    NOT_REQUIRED, dated_date: DatedDate [FIX44..],
    NOT_REQUIRED, interest_accrual_date: InterestAccrualDate [FIX44..],
    NOT_REQUIRED, no_instrument_parties: NoInstrumentParties [FIX50..],
    NOT_REQUIRED, no_complex_events: NoComplexEvents [FIX50SP2..],
});

define_message!(InstrumentLeg {
    REQUIRED, leg_symbol: LegSymbol,
    NOT_REQUIRED, leg_symbol_sfx: LegSymbolSfx,
    NOT_REQUIRED, leg_security_id: LegSecurityID,
    NOT_REQUIRED, leg_security_id_source: LegSecurityIDSource => REQUIRED_WHEN |message: &InstrumentLeg,_| { !message.leg_security_id.is_empty() },
    NOT_REQUIRED, no_leg_security_alt_id: NoLegSecurityAltID,
    NOT_REQUIRED, leg_product: LegProduct,
    NOT_REQUIRED, leg_cfi_code: LegCFICode,
    NOT_REQUIRED, leg_security_type: LegSecurityType => REQUIRED_WHEN |message: &InstrumentLeg,_| { !message.leg_security_sub_type.is_empty() },
    NOT_REQUIRED, leg_security_sub_type: LegSecuritySubType,
    NOT_REQUIRED, leg_maturity_month_year: LegMaturityMonthYear,
    NOT_REQUIRED, leg_maturity_date: LegMaturityDate,
    NOT_REQUIRED, leg_maturity_time: LegMaturityTime,
    NOT_REQUIRED, leg_coupon_payment_date: LegCouponPaymentDate,
    NOT_REQUIRED, leg_issue_date: LegIssueDate,
    NOT_REQUIRED, leg_repo_collateral_security_type: LegRepoCollateralSecurityType,
    NOT_REQUIRED, leg_repurchase_term: LegRepurchaseTerm,
    NOT_REQUIRED, leg_repurchase_rate: LegRepurchaseRate,
    NOT_REQUIRED, leg_factor: LegFactor,
    NOT_REQUIRED, leg_credit_rating: LegCreditRating,
    NOT_REQUIRED, leg_instr_registry: LegInstrRegistry,
    NOT_REQUIRED, leg_country_of_issue: LegCountryOfIssue,
    NOT_REQUIRED, leg_state_or_province_of_issue: LegStateOrProvinceOfIssue,
    NOT_REQUIRED, leg_locale_of_issue: LegLocaleOfIssue,
    NOT_REQUIRED, leg_redemption_date: LegRedemptionDate,
    NOT_REQUIRED, leg_strike_price: LegStrikePrice,
    NOT_REQUIRED, leg_strike_currency: LegStrikeCurrency,
    NOT_REQUIRED, leg_opt_attribute: LegOptAttribute,
    NOT_REQUIRED, leg_contract_multiplier: LegContractMultiplier,
    NOT_REQUIRED, leg_contract_multiplier_unit: LegContractMultiplierUnit,
    NOT_REQUIRED, leg_flow_schedule_type: LegFlowScheduleType,
    NOT_REQUIRED, leg_unit_of_measure: LegUnitOfMeasure,
    NOT_REQUIRED, leg_unit_of_measure_qty: LegUnitOfMeasureQty,
    NOT_REQUIRED, leg_price_unit_of_measure: LegPriceUnitOfMeasure,
    NOT_REQUIRED, leg_price_unit_of_measure_qty: LegPriceUnitOfMeasureQty,
    NOT_REQUIRED, leg_time_unit: LegTimeUnit,
    NOT_REQUIRED, leg_exercise_style: LegExerciseStyle,
    NOT_REQUIRED, leg_coupon_rate: LegCouponRate,
    NOT_REQUIRED, leg_security_exchange: LegSecurityExchange,
    NOT_REQUIRED, leg_issuer: LegIssuer,
    NOT_REQUIRED, encoded_leg_issuer_len: EncodedLegIssuerLen,
    NOT_REQUIRED, encoded_leg_issuer: EncodedLegIssuer,
    NOT_REQUIRED, leg_security_desc: LegSecurityDesc,
    NOT_REQUIRED, encoded_leg_security_desc_len: EncodedLegSecurityDescLen,
    NOT_REQUIRED, encoded_leg_security_desc: EncodedLegSecurityDesc,
    NOT_REQUIRED, leg_ratio_qty: LegRatioQty,
    NOT_REQUIRED, leg_side: LegSide,
    NOT_REQUIRED, leg_currency: LegCurrency,
    NOT_REQUIRED, leg_poll: LegPool,
    NOT_REQUIRED, leg_dated_date: LegDatedDate,
    NOT_REQUIRED, leg_contract_settl_month: LegContractSettlMonth,
    NOT_REQUIRED, leg_interest_accrual_date: LegInterestAccrualDate,
    NOT_REQUIRED, leg_put_or_call: LegPutOrCall,
    NOT_REQUIRED, leg_option_ratio: LegOptionRatio,
    NOT_REQUIRED, leg_price: LegPrice,
});

define_message!(InstrumentParty {
    REQUIRED, instrument_party_id: InstrumentPartyID [FIX50..],
    REQUIRED, instrument_party_id_source: InstrumentPartyIDSource [FIX50..], //Conditionally required if InstrumentPartyID is specified, but InstrumentPartyID is required, so this is also required.
    NOT_REQUIRED, instrument_party_role: InstrumentPartyRole [FIX50..],
    NOT_REQUIRED, no_instrument_party_sub_ids: NoInstrumentPartySubIDs [FIX50..],
});

define_message!(InstrumentPtysSubGrp {
    REQUIRED, instrument_party_sub_id: InstrumentPartySubID [FIX50..],
    REQUIRED, instrument_party_sub_id_type: InstrumentPartySubIDType [FIX50..],
});

define_message!(LegSecAltIDGrp {
    REQUIRED, leg_security_alt_id: LegSecurityAltID,
    REQUIRED, leg_security_alt_id_source: LegSecurityAltIDSource,
});

define_message!(LinesOfTextGrp {
    REQUIRED, text: Text [FIX40..],
    NOT_REQUIRED, encoded_text_len: EncodedTextLen [FIX42..],
    NOT_REQUIRED, encoded_text: EncodedText [FIX42..],
});

define_message!(MsgTypeGrp {
    REQUIRED, ref_msg_type: RefMsgType [FIX42..],
    REQUIRED, msg_direction: MsgDirection [FIX42..],
    NOT_REQUIRED, ref_appl_ver_id: RefApplVerID [FIX50..],
    NOT_REQUIRED, ref_appl_ext_id: RefApplExtID [FIX50..],
    NOT_REQUIRED, ref_cstm_appl_ver_id: RefCstmApplVerID [FIX50..],
    NOT_REQUIRED, default_ver_indicator: DefaultVerIndicator [FIX50SP1..],
});

define_message!(Order {
    REQUIRED, cl_ord_id: ClOrdID,
    NOT_REQUIRED, allocs: NoAllocs,
});

define_message!(RateSourceGrp {
    REQUIRED, rate_source: RateSource,
    REQUIRED, rate_source_type: RateSourceType,
    NOT_REQUIRED, reference_page: ReferencePage => REQUIRED_WHEN |message: &RateSourceGrp,_| { message.rate_source == other_field_types::RateSource::Other },
});

define_message!(RoutingGrp {
    REQUIRED, routing_type: RoutingType [FIX42..],
    REQUIRED, routing_id: RoutingID [FIX42..],
});

define_message!(SecAltIDGrp {
    REQUIRED, security_alt_id: SecurityAltID [FIX43..],
    REQUIRED, security_alt_id_source: SecurityAltIDSource [FIX43..],
});

define_message!(UnderlyingInstrument {
    REQUIRED, underlying_symbol: UnderlyingSymbol,
    NOT_REQUIRED, underlying_symbol_sfx: UnderlyingSymbolSfx,
    NOT_REQUIRED, underlying_security_id: UnderlyingSecurityID,
    NOT_REQUIRED, underlying_security_id_source: UnderlyingSecurityIDSource => REQUIRED_WHEN |message: &UnderlyingInstrument,_| { !message.underlying_security_id.is_empty() },
    NOT_REQUIRED, no_underlying_security_alt_id: NoUnderlyingSecurityAltID,
    NOT_REQUIRED, underlying_product: UnderlyingProduct,
    NOT_REQUIRED, underlying_cfi_code: UnderlyingCFICode,
    NOT_REQUIRED, underlying_security_type: UnderlyingSecurityType => REQUIRED_WHEN |message: &UnderlyingInstrument,_| { !message.underlying_security_sub_type.is_empty() },
    NOT_REQUIRED, underlying_security_sub_type: UnderlyingSecuritySubType,
    NOT_REQUIRED, underlying_maturity_month_year: UnderlyingMaturityMonthYear,
    NOT_REQUIRED, underlying_maturity_date: UnderlyingMaturityDate,
    NOT_REQUIRED, underlying_maturity_time: UnderlyingMaturityTime,
    NOT_REQUIRED, underlying_coupon_payment_date: UnderlyingCouponPaymentDate,
    NOT_REQUIRED, underlying_restructuring_type: UnderlyingRestructuringType,
    NOT_REQUIRED, underlying_seniority: UnderlyingSeniority,
    NOT_REQUIRED, underlying_notional_percentage_outstanding: UnderlyingNotionalPercentageOutstanding,
    NOT_REQUIRED, underlying_original_notional_percentage_outstanding: UnderlyingOriginalNotionalPercentageOutstanding,
    NOT_REQUIRED, underlying_attachment_point: UnderlyingAttachmentPoint,
    NOT_REQUIRED, underlying_detachment_point: UnderlyingDetachmentPoint,
    NOT_REQUIRED, underlying_issue_date: UnderlyingIssueDate,
    NOT_REQUIRED, underlying_repo_collateral_security_type: UnderlyingRepoCollateralSecurityType,
    NOT_REQUIRED, underlying_repurchase_term: UnderlyingRepurchaseTerm,
    NOT_REQUIRED, underlying_repurchase_rate: UnderlyingRepurchaseRate,
    NOT_REQUIRED, underlying_factor: UnderlyingFactor,
    NOT_REQUIRED, underlying_credit_rating: UnderlyingCreditRating,
    NOT_REQUIRED, underlying_instr_registry: UnderlyingInstrRegistry,
    NOT_REQUIRED, underlying_country_of_issue: UnderlyingCountryOfIssue,
    NOT_REQUIRED, underlying_state_or_province_of_issue: UnderlyingStateOrProvinceOfIssue,
    NOT_REQUIRED, underlying_locale_of_issue: UnderlyingLocaleOfIssue,
    NOT_REQUIRED, underlying_redemption_date: UnderlyingRedemptionDate,
    NOT_REQUIRED, underlying_strike_price: UnderlyingStrikePrice,
    NOT_REQUIRED, underlying_strike_currency: UnderlyingStrikeCurrency,
    NOT_REQUIRED, underlying_opt_attribute: UnderlyingOptAttribute,
    NOT_REQUIRED, underlying_contract_multiplier: UnderlyingContractMultiplier,
    NOT_REQUIRED, underlying_contract_multiplier_unit: UnderlyingContractMultiplierUnit,
    NOT_REQUIRED, underlying_flow_schedule_type: UnderlyingFlowScheduleType,
    NOT_REQUIRED, underlying_unit_of_measure: UnderlyingUnitOfMeasure,
    NOT_REQUIRED, underlying_unit_of_measure_qty: UnderlyingUnitOfMeasureQty,
    NOT_REQUIRED, underlying_price_unit_of_measure: UnderlyingPriceUnitOfMeasure,
    NOT_REQUIRED, underlying_price_unit_of_measure_qty: UnderlyingPriceUnitOfMeasureQty,
    NOT_REQUIRED, underlying_time_unit: UnderlyingTimeUnit,
    NOT_REQUIRED, underlying_exercise_style: UnderlyingExerciseStyle,
    NOT_REQUIRED, underlying_coupon_rate: UnderlyingCouponRate,
    NOT_REQUIRED, underlying_security_exchange: UnderlyingSecurityExchange,
    NOT_REQUIRED, underlying_issuer: UnderlyingIssuer,
    NOT_REQUIRED, encoded_underlying_issuer_len: EncodedUnderlyingIssuerLen,
    NOT_REQUIRED, encoded_underlying_issuer: EncodedUnderlyingIssuer,
    NOT_REQUIRED, underlying_security_desc: UnderlyingSecurityDesc,
    NOT_REQUIRED, encoded_underlying_security_desc_len: EncodedUnderlyingSecurityDescLen,
    NOT_REQUIRED, encoded_underlying_security_desc: EncodedUnderlyingSecurityDesc,
    NOT_REQUIRED, underlying_cp_program: UnderlyingCPProgram,
    NOT_REQUIRED, underlying_cp_reg_type: UnderlyingCPRegType,
    NOT_REQUIRED, underlying_allocation_percent: UnderlyingAllocationPercent,
    NOT_REQUIRED, underlying_currency: UnderlyingCurrency,
    NOT_REQUIRED, underlying_qty: UnderlyingQty,
    NOT_REQUIRED, underlying_settlement_type: UnderlyingSettlementType,
    NOT_REQUIRED, underlying_cash_amount: UnderlyingCashAmount,
    NOT_REQUIRED, underlying_cash_type: UnderlyingCashType,
    NOT_REQUIRED, underlying_px: UnderlyingPx,
    NOT_REQUIRED, underlying_dirty_price: UnderlyingDirtyPrice,
    NOT_REQUIRED, underlying_end_price: UnderlyingEndPrice,
    NOT_REQUIRED, underlying_start_value: UnderlyingStartValue,
    NOT_REQUIRED, underlying_current_value: UnderlyingCurrentValue,
    NOT_REQUIRED, underlying_end_value: UnderlyingEndValue,
    NOT_REQUIRED, no_underlying_stips: NoUnderlyingStips,
    NOT_REQUIRED, underlying_adjusted_quantity: UnderlyingAdjustedQuantity,
    NOT_REQUIRED, underlying_fx_rate: UnderlyingFXRate,
    NOT_REQUIRED, underlying_fx_rate_calc: UnderlyingFXRateCalc,
    NOT_REQUIRED, underlying_cap_value: UnderlyingCapValue,
    NOT_REQUIRED, no_undly_instrument_parties: NoUndlyInstrumentParties,
    NOT_REQUIRED, underlying_settl_method: UnderlyingSettlMethod,
    NOT_REQUIRED, underlying_put_or_call: UnderlyingPutOrCall,
});

define_message!(UndlyInstrumentPtysSubGrp {
    REQUIRED, underlying_instrument_party_sub_id: UnderlyingInstrumentPartySubID,
    REQUIRED, underlying_instrument_party_sub_id_type: UnderlyingInstrumentPartySubIDType,
});

define_message!(UnderlyingStipulation {
    REQUIRED, underlying_stip_type: UnderlyingStipType,
    REQUIRED, underlying_stip_value: UnderlyingStipValue,
});

define_message!(UndSecAltIDGrp {
    REQUIRED, underlying_security_alt_id: UnderlyingSecurityAltID,
    REQUIRED, underlying_security_alt_id_source: UnderlyingSecurityAltIDSource,
});

