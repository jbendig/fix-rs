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

use crate::dictionary::field_types::generic::{BoolTrueOrBlankFieldType,CharFieldType,CountryFieldType,CurrencyFieldType,DataFieldType,DayOfMonthFieldType,IntFieldType,LengthFieldType,LocalMktDateFieldType,MonthYearFieldType,NoneFieldType,RepeatingGroupFieldType,SeqNumFieldType,StringFieldType,UTCTimeOnlyFieldType,UTCTimestampFieldType};
use crate::dictionary::field_types::other as other_field_types;
use crate::dictionary::field_types::other::{ApplVerIDFieldType,BusinessRejectReasonFieldType,ComplexEventConditionFieldType,ComplexEventPriceBoundaryMethodFieldType,ComplexEventPriceTimeTypeFieldType,ComplexEventTypeFieldType,ContractMultiplierUnitFieldType,CPProgramFieldType,DefaultApplVerIDFieldType,EmailTypeFieldType,EncryptMethodFieldType,EventTypeFieldType,ExerciseStyleFieldType,FlowScheduleTypeFieldType,HandlInstFieldType,InstrmtAssignmentMethodFieldType,IssuerFieldType,ListMethodFieldType,MsgDirectionFieldType,NotRequiredSecurityIDSourceFieldType,NotRequiredSecurityTypeFieldType as SecurityTypeFieldType,NotRequiredSideFieldType,NotRequiredSymbolSfxFieldType as SymbolSfxFieldType,NotRequiredTimeUnitFieldType as TimeUnitFieldType,OptPayoutTypeFieldType,OrdTypeFieldType,PartyIDSourceFieldType,PartyRoleFieldType,PartySubIDTypeFieldType,PriceQuoteMethodFieldType,ProductFieldType,PutOrCallFieldType,RateSourceFieldType,RateSourceTypeFieldType,RequiredSecurityIDSourceFieldType,RequiredSideFieldType,RequiredStipulationTypeFieldType as StipulationTypeFieldType,RestructuringTypeFieldType,RoutingTypeFieldType,SecurityStatusFieldType,SeniorityFieldType,SessionRejectReasonFieldType,SettlMethodFieldType,SettlTypeFieldType,StrikePriceBoundaryMethodFieldType,StrikePriceDeterminationMethodFieldType,TimeInForceFieldType,UnderlyingCashTypeFieldType,UnderlyingFXRateCalcFieldType,UnderlyingPriceDeterminationMethodFieldType,UnderlyingSettlementTypeFieldType,UnitOfMeasureFieldType,ValuationMethodFieldType};
use crate::field_tag;
use crate::fix_version::FIXVersion;
use crate::message::{REQUIRED,NOT_REQUIRED};
use crate::rule::Rule;

//TODO: Create implementations for all of these types.
type PercentageFieldType = StringFieldType;
type PriceFieldType = StringFieldType;
type TZTimeOnlyFieldType = StringFieldType;
type AmtFieldType = StringFieldType;
type QtyFieldType = StringFieldType;
type ExchangeFieldType = StringFieldType; //See ISO 10383 for a complete list: https://www.iso20022.org/10383/iso-10383-market-identifier-codes

define_fields!(
    Account: StringFieldType = 1,
    BeginSeqNo: SeqNumFieldType = 7,
    ClOrdID: StringFieldType = 11,
    Currency: CurrencyFieldType = 15,
    EndSeqNo: SeqNumFieldType = 16,
    HandlInst: HandlInstFieldType = 21,
    SecurityIDSource: NotRequiredSecurityIDSourceFieldType = 22,
    NoLinesOfText: RepeatingGroupFieldType<LinesOfTextGrp> = 33,
    MsgSeqNum: SeqNumFieldType = 34, //TODO: Special field probably might be better off built into the parser.
    NewSeqNo: SeqNumFieldType = 36,
    OrderID: StringFieldType = 37,
    OrderQty: StringFieldType = 38, //Qty
    OrdType: OrdTypeFieldType = 40,
    OrigTime: UTCTimestampFieldType = 42,
    PossDupFlag: BoolTrueOrBlankFieldType = 43,
    Price: StringFieldType = 44, //Price
    RefSeqNum: SeqNumFieldType = 45,
    RelatedSym: StringFieldType = 46, //TODO: Old special field that can be repeated without a repeating group.
    SecurityID: StringFieldType = 48,
    SenderCompID: StringFieldType = 49,
    SenderSubID: StringFieldType = 50,
    SendingTime: UTCTimestampFieldType = 52,
    SideField: RequiredSideFieldType = 54,
    Symbol: StringFieldType = 55,
    TargetCompID: StringFieldType = 56,
    TargetSubID: StringFieldType = 57,
    Text: StringFieldType = 58,
    TimeInForce: TimeInForceFieldType = 59,
    TransactTime: UTCTimestampFieldType = 60,
    SettlType: SettlTypeFieldType = 63,
    SettlDate: LocalMktDateFieldType = 64,
    SymbolSfx: SymbolSfxFieldType = 65,
    //NoOrders: RepeatingGroupFieldType<Order> = 73, //TODO: The repeating group type here depends on the message using NoOrders.
    //NoAllocs: RepeatingGroupFieldType<Alloc> = 78, //TODO: The repeating group type here depends on the message using NoAllocs.
    AllocAccount: StringFieldType = 79,
    Signature: DataFieldType = 89 => Rule::ConfirmPreviousTag{ previous_tag: SignatureLength::tag() },
    SecureDataLen: NoneFieldType = 90 => Rule::PrepareForBytes{ bytes_tag: SecureData::tag() },
    SecureData: DataFieldType = 91 => Rule::ConfirmPreviousTag{ previous_tag: SecureDataLen::tag() },
    SignatureLength: NoneFieldType = 93 => Rule::PrepareForBytes{ bytes_tag: Signature::tag() },
    EmailType: EmailTypeFieldType = 94,
    RawDataLength: NoneFieldType = 95 => Rule::PrepareForBytes{ bytes_tag: RawData::tag() },
    RawData: DataFieldType = 96 => Rule::ConfirmPreviousTag{ previous_tag: RawDataLength::tag() },
    PossResend: StringFieldType = 97, //Bool
    EncryptMethod: EncryptMethodFieldType = 98,
    Issuer: IssuerFieldType = 106,
    SecurityDesc: StringFieldType = 107,
    HeartBtInt: IntFieldType = 108,
    MinQty: StringFieldType = 110, //Qty
    MaxFloor: StringFieldType = 111, //Qty
    TestReqID: StringFieldType = 112,
    OnBehalfOfCompID: StringFieldType = 115,
    OnBehalfOfSubID: StringFieldType = 116,
    OrigSendingTime: UTCTimestampFieldType = 122,
    GapFillFlag: BoolTrueOrBlankFieldType = 123,
    DeliverToCompID: StringFieldType = 128,
    DeliverToSubID: StringFieldType = 129,
    BidSize: StringFieldType = 134, //Qty
    ResetSeqNumFlag: BoolTrueOrBlankFieldType = 141,
    SenderLocationID: StringFieldType = 142,
    TargetLocationID: StringFieldType = 143,
    OnBehalfOfLocationID: StringFieldType = 144,
    DeliverToLocationID: StringFieldType = 145,
    NoRelatedSym: RepeatingGroupFieldType<Instrument> = 146,
    Subject: StringFieldType = 147,
    CashOrderQty: StringFieldType = 152, //Qty
    EmailThreadID: StringFieldType = 164,
    SecurityType: SecurityTypeFieldType = 167,
    MaturityMonthYear: MonthYearFieldType = 200,
    PutOrCall: PutOrCallFieldType = 201,
    StrikePrice: StringFieldType = 202, //Price
    MaturityDay: DayOfMonthFieldType = 205,
    OptAttribute: CharFieldType = 206,
    SecurityExchange: ExchangeFieldType = 207,
    XmlDataLen: NoneFieldType = 212 => Rule::PrepareForBytes{ bytes_tag: XmlData::tag() },
    XmlData: DataFieldType = 213 => Rule::ConfirmPreviousTag{ previous_tag: XmlDataLen::tag() },
    NoRoutingIDs: RepeatingGroupFieldType<RoutingGrp> = 215,
    RoutingType: RoutingTypeFieldType = 216,
    RoutingID: StringFieldType = 217,
    CouponRate: PercentageFieldType = 223,
    CouponPaymentDate: LocalMktDateFieldType = 224, //TODO: Use UTCDate when FIX version < 4.4.
    IssueDate: LocalMktDateFieldType = 225, //TODO: Use UTCDate when FIX version < 4.4.
    RepurchaseTerm: IntFieldType = 226,
    RepurchaseRate: PercentageFieldType = 227,
    Factor: StringFieldType = 228, //Float
    ContractMultiplier: StringFieldType = 231, //Float
    RepoCollateralSecurityType: SecurityTypeFieldType = 239,
    RedemptionDate: LocalMktDateFieldType = 240,
    UnderlyingCouponPaymentDate: LocalMktDateFieldType = 241,
    UnderlyingIssueDate: LocalMktDateFieldType = 242,
    UnderlyingRepoCollateralSecurityType: SecurityTypeFieldType = 243,
    UnderlyingRepurchaseTerm: IntFieldType = 244,
    UnderlyingRepurchaseRate: PercentageFieldType = 245,
    UnderlyingFactor: StringFieldType = 246, //Float
    UnderlyingRedemptionDate: LocalMktDateFieldType = 247,
    LegCouponPaymentDate: LocalMktDateFieldType = 248,
    LegIssueDate: LocalMktDateFieldType = 249,
    LegRepoCollateralSecurityType: SecurityTypeFieldType = 250,
    LegRepurchaseTerm: IntFieldType = 251,
    LegRepurchaseRate: PercentageFieldType = 252,
    LegFactor: StringFieldType = 253, //Float
    LegRedemptionDate: LocalMktDateFieldType = 254,
    CreditRating: StringFieldType = 255,
    UnderlyingCreditRating: StringFieldType = 256,
    LegCreditRating: StringFieldType = 257,
    UnderlyingSecurityIDSource: NotRequiredSecurityIDSourceFieldType = 305,
    UnderlyingIssuer: IssuerFieldType = 306,
    UnderlyingSecurityDesc: StringFieldType = 307,
    UnderlyingSecurityExchange: ExchangeFieldType = 308,
    UnderlyingSecurityID: StringFieldType = 309,
    UnderlyingSecurityType: SecurityTypeFieldType = 310,
    UnderlyingSymbol: StringFieldType = 311,
    UnderlyingSymbolSfx: SymbolSfxFieldType = 312,
    UnderlyingMaturityMonthYear: MonthYearFieldType = 313,
    UnderlyingPutOrCall: PutOrCallFieldType = 315,
    UnderlyingStrikePrice: StringFieldType = 316, //Price
    UnderlyingOptAttribute: CharFieldType = 317,
    UnderlyingCurrency: CurrencyFieldType = 318,
    MessageEncoding: StringFieldType = 347,
    EncodedIssuerLen: NoneFieldType = 348 => Rule::PrepareForBytes{ bytes_tag: EncodedIssuer::tag() },
    EncodedIssuer: DataFieldType = 349 => Rule::ConfirmPreviousTag{ previous_tag: EncodedIssuerLen::tag() },
    EncodedSecurityDescLen: NoneFieldType = 350 => Rule::PrepareForBytes{ bytes_tag: EncodedSecurityDesc::tag() },
    EncodedSecurityDesc: DataFieldType = 351 => Rule::ConfirmPreviousTag{ previous_tag: EncodedSecurityDescLen::tag() },
    EncodedTextLen: NoneFieldType = 354 => Rule::PrepareForBytes{ bytes_tag: EncodedText::tag() },
    EncodedText: DataFieldType = 355 => Rule::ConfirmPreviousTag{ previous_tag: EncodedTextLen::tag() },
    EncodedSubjectLen: NoneFieldType = 356 => Rule::PrepareForBytes{ bytes_tag: EncodedSubject::tag() },
    EncodedSubject: DataFieldType = 357 => Rule::ConfirmPreviousTag{ previous_tag: EncodedSubjectLen::tag() },
    EncodedUnderlyingIssuerLen: NoneFieldType = 362 => Rule::PrepareForBytes{ bytes_tag: EncodedUnderlyingIssuer::tag() },
    EncodedUnderlyingIssuer: DataFieldType = 363 => Rule::ConfirmPreviousTag{ previous_tag: EncodedUnderlyingIssuerLen::tag() },
    EncodedUnderlyingSecurityDescLen: NoneFieldType = 364 => Rule::PrepareForBytes{ bytes_tag: EncodedUnderlyingSecurityDesc::tag() },
    EncodedUnderlyingSecurityDesc: DataFieldType = 365 => Rule::ConfirmPreviousTag{ previous_tag: EncodedUnderlyingSecurityDescLen::tag() },
    LastMsgSeqNumProcessed: SeqNumFieldType = 369,
    OnBehalfOfSendingTime: UTCTimestampFieldType = 370,
    RefTagID: StringFieldType = 371, //int
    RefMsgType: StringFieldType = 372,
    SessionRejectReason: SessionRejectReasonFieldType = 373,
    BusinessRejectRefID: StringFieldType = 379,
    BusinessRejectReason: BusinessRejectReasonFieldType = 380,
    MaxMessageSize: LengthFieldType = 383,
    NoMsgTypeGrp: RepeatingGroupFieldType<MsgTypeGrp> = 384,
    MsgDirection: MsgDirectionFieldType = 385,
    UnderlyingCouponRate: PercentageFieldType = 435,
    UnderlyingContractMultiplier: StringFieldType = 436, //Float
    NoSecurityAltID: RepeatingGroupFieldType<SecAltIDGrp> = 454,
    SecurityAltID: StringFieldType = 455,
    SecurityAltIDSource: RequiredSecurityIDSourceFieldType = 456,
    NoUnderlyingSecurityAltID: RepeatingGroupFieldType<UndSecAltIDGrp> = 457,
    UnderlyingSecurityAltID: StringFieldType = 458,
    UnderlyingSecurityAltIDSource: RequiredSecurityIDSourceFieldType = 459,
    Product: ProductFieldType = 460,
    CFICode: StringFieldType = 461,
    UnderlyingProduct: ProductFieldType = 462,
    UnderlyingCFICode: StringFieldType = 463,
    TestMessageIndicator: StringFieldType = 464, //Bool
    CountryOfIssue: CountryFieldType = 470,
    StateOrProvinceOfIssue: StringFieldType = 471,
    LocaleOfIssue: StringFieldType = 472, //Full code list is available for purchase here: http://www.iata.org/publications/store/Pages/airline-coding-directory.aspx
    MaturityDate: LocalMktDateFieldType = 541,
    UnderlyingMaturityDate: LocalMktDateFieldType = 542,
    InstrRegistry: StringFieldType = 543,
    Username: StringFieldType = 553,
    Password: StringFieldType = 554,
    NoLegs: RepeatingGroupFieldType<InstrumentLeg> = 555,
    LegCurrency: CurrencyFieldType = 556,
    LegPrice: PriceFieldType = 566,
    UnderlyingCountryOfIssue: CountryFieldType = 592,
    UnderlyingStateOrProvinceOfIssue: StringFieldType = 593,
    UnderlyingLocaleOfIssue: StringFieldType = 594, //See LocaleOfIssue (472).
    UnderlyingInstrRegistry: StringFieldType = 595,
    LegCountryOfIssue: CountryFieldType = 596,
    LegStateOrProvinceOfIssue: StringFieldType = 597,
    LegLocaleOfIssue: StringFieldType = 598, //See LocaleOfIssue (472).
    LegInstrRegistry: StringFieldType = 599,
    LegSymbol: StringFieldType = 600,
    LegSymbolSfx: SymbolSfxFieldType = 601,
    LegSecurityID: StringFieldType = 602,
    LegSecurityIDSource: NotRequiredSecurityIDSourceFieldType = 603,
    NoLegSecurityAltID: RepeatingGroupFieldType<LegSecAltIDGrp> = 604,
    LegSecurityAltID: StringFieldType = 605,
    LegSecurityAltIDSource: RequiredSecurityIDSourceFieldType = 606,
    LegProduct: ProductFieldType = 607,
    LegCFICode: StringFieldType = 608,
    LegSecurityType: SecurityTypeFieldType = 609,
    LegMaturityMonthYear: MonthYearFieldType = 610,
    LegMaturityDate: LocalMktDateFieldType = 611,
    LegStrikePrice: PriceFieldType = 612,
    LegOptAttribute: CharFieldType = 613,
    LegContractMultiplier: StringFieldType = 614, //Float
    LegCouponRate: PercentageFieldType = 615,
    LegSecurityExchange: ExchangeFieldType = 616,
    LegIssuer: IssuerFieldType = 617,
    EncodedLegIssuerLen: NoneFieldType = 618 => Rule::PrepareForBytes{ bytes_tag: EncodedLegIssuer::tag() },
    EncodedLegIssuer: DataFieldType = 619 => Rule::ConfirmPreviousTag{ previous_tag: EncodedLegIssuerLen::tag() },
    LegSecurityDesc: StringFieldType = 620,
    EncodedLegSecurityDescLen: NoneFieldType = 621 => Rule::PrepareForBytes{ bytes_tag: EncodedLegSecurityDesc::tag() },
    EncodedLegSecurityDesc: DataFieldType = 622 => Rule::ConfirmPreviousTag{ previous_tag: EncodedLegSecurityDescLen::tag() },
    LegRatioQty: StringFieldType = 623, //Float
    LegSide: NotRequiredSideFieldType = 624,
    NoHops: RepeatingGroupFieldType<HopGrp> = 627,
    HopCompID: StringFieldType = 628,
    HopSendingTime: UTCTimestampFieldType = 629,
    HopRefID: SeqNumFieldType = 630,
    ContractSettlMonth: MonthYearFieldType = 667,
    Pool: StringFieldType = 691,
    NoUnderlyings: RepeatingGroupFieldType<UnderlyingInstrument> = 711,
    LegDatedDate: LocalMktDateFieldType = 739,
    LegPool: StringFieldType = 740,
    SecuritySubType: StringFieldType = 762,
    UnderlyingSecuritySubType: StringFieldType = 763,
    LegSecuritySubType: StringFieldType = 764,
    NextExpectedMsgSeqNum: SeqNumFieldType = 789,
    UnderlyingPx: PriceFieldType = 810,
    NoEvents: RepeatingGroupFieldType<EvntGrp> = 864,
    EventType: EventTypeFieldType = 865,
    EventDate: LocalMktDateFieldType = 866,
    EventPx: PriceFieldType = 867,
    EventText: StringFieldType = 868,
    DatedDate: LocalMktDateFieldType = 873,
    InterestAccrualDate: LocalMktDateFieldType = 874,
    CPProgram: CPProgramFieldType = 875,
    CPRegType: StringFieldType = 876,
    UnderlyingCPProgram: CPProgramFieldType = 877,
    UnderlyingCPRegType: StringFieldType = 878,
    UnderlyingQty: QtyFieldType = 879,
    UnderlyingDirtyPrice: PriceFieldType = 882,
    UnderlyingEndPrice: PriceFieldType = 883,
    UnderlyingStartValue: AmtFieldType = 884,
    UnderlyingCurrentValue: AmtFieldType = 885,
    UnderlyingEndValue: AmtFieldType = 886,
    NoUnderlyingStips: RepeatingGroupFieldType<UnderlyingStipulation> = 887,
    UnderlyingStipType: StipulationTypeFieldType = 888,
    UnderlyingStipValue: StringFieldType = 889, //TODO: Parsable expression.
    NewPassword: StringFieldType = 925,
    UnderlyingStrikeCurrency: CurrencyFieldType = 941,
    LegStrikeCurrency: CurrencyFieldType = 942,
    StrikeCurrency: CurrencyFieldType = 947,
    LegContractSettlMonth: MonthYearFieldType = 955,
    LegInterestAccrualDate: LocalMktDateFieldType = 956,
    SecurityStatus: SecurityStatusFieldType = 965,
    SettleOnOpenFlag: StringFieldType = 966,
    StrikeMultiplier: StringFieldType = 967, //Float
    StrikeValue: StringFieldType = 968, //Float
    MinPriceIncrement: StringFieldType = 969, //Float
    PositionLimit: IntFieldType = 970,
    NTPositionLimit: IntFieldType = 971,
    UnderlyingAllocationPercent: PercentageFieldType = 972,
    UnderlyingCashAmount: AmtFieldType = 973,
    UnderlyingCashType: UnderlyingCashTypeFieldType = 974,
    UnderlyingSettlementType: UnderlyingSettlementTypeFieldType = 975,
    UnitOfMeasure: UnitOfMeasureFieldType = 996,
    TimeUnit: TimeUnitFieldType = 997,
    UnderlyingUnitOfMeasure: UnitOfMeasureFieldType = 998,
    LegUnitOfMeasure: UnitOfMeasureFieldType = 999,
    UnderlyingTimeUnit: TimeUnitFieldType = 1000,
    LegTimeUnit: TimeUnitFieldType = 1001,
    LegOptionRatio: StringFieldType = 1017, //Float
    NoInstrumentParties: RepeatingGroupFieldType<InstrumentParty> = 1018,
    InstrumentPartyID: StringFieldType = 1019, //Valid PartyID values are dependent on PartyIDSource and PartyRole.
    UnderlyingDeliveryAmount: AmtFieldType = 1037,
    UnderlyingCapValue: AmtFieldType = 1038,
    UnderlyingSettlMethod: StringFieldType = 1039,
    UnderlyingAdjustedQuantity: StringFieldType = 1044, //Qty
    UnderlyingFXRate: StringFieldType = 1045, //Float
    UnderlyingFXRateCalc: UnderlyingFXRateCalcFieldType = 1046,
    NoUndlyInstrumentParties: RepeatingGroupFieldType<UndlyInstrumentPtysSubGrp> = 1058,
    InstrmtAssignmentMethod: InstrmtAssignmentMethodFieldType = 1049,
    InstrumentPartyIDSource: PartyIDSourceFieldType = 1050,
    InstrumentPartyRole: PartyRoleFieldType = 1051,
    NoInstrumentPartySubIDs: RepeatingGroupFieldType<InstrumentPtysSubGrp> = 1052,
    InstrumentPartySubID: StringFieldType = 1053,
    InstrumentPartySubIDType: PartySubIDTypeFieldType = 1054,
    NoUndlyInstrumentPartySubIDs: RepeatingGroupFieldType<UndlyInstrumentPtysSubGrp> = 1062,
    UnderlyingInstrumentPartySubID: StringFieldType = 1063,
    UnderlyingInstrumentPartySubIDType: PartySubIDTypeFieldType = 1064,
    MaturityTime: TZTimeOnlyFieldType = 1079,
    ApplVerID: ApplVerIDFieldType = 1128 => Rule::RequiresFIXVersion{ fix_version: FIXVersion::FIXT_1_1 },
    CstmApplVerID: StringFieldType = 1129,
    RefApplVerID: ApplVerIDFieldType = 1130,
    RefCstmApplVerID: StringFieldType = 1131,
    DefaultApplVerID: DefaultApplVerIDFieldType = 1137,
    EventTime: UTCTimestampFieldType = 1145,
    MinPriceIncrementAmount: AmtFieldType = 1146,
    UnitOfMeasureQty: StringFieldType = 1147, //Qty
    SecurityGroup: StringFieldType = 1151,
    ApplExtID: StringFieldType = 1156, //int
    SecurityXMLLen: NoneFieldType = 1184 => Rule::PrepareForBytes{ bytes_tag: SecurityXML::tag() },
    SecurityXML: DataFieldType = 1185 => Rule::ConfirmPreviousTag{ previous_tag: SecurityXMLLen::tag() },
    SecurityXMLSchema: StringFieldType = 1186,
    PriceUnitOfMeasure: UnitOfMeasureFieldType = 1191,
    PriceUnitOfMeasureQty: StringFieldType = 1192, //Qty
    SettlMethod: SettlMethodFieldType = 1193,
    ExerciseStyle: ExerciseStyleFieldType = 1194,
    OptPayoutAmount: AmtFieldType = 1195,
    PriceQuoteMethod: PriceQuoteMethodFieldType = 1196,
    ValuationMethod: ValuationMethodFieldType = 1197,
    ListMethod: ListMethodFieldType = 1198,
    CapPrice: PriceFieldType = 1199,
    FloorPrice: PriceFieldType = 1200,
    LegMaturityTime: TZTimeOnlyFieldType = 1212,
    UnderlyingMaturityTime: TZTimeOnlyFieldType = 1213,
    LegUnitOfMeasureQty: StringFieldType = 1224, //Qty
    ProductComplex: StringFieldType = 1227,
    FlexibleProductElgibilityIndicator: BoolTrueOrBlankFieldType = 1242,
    FlexibleIndicator: BoolTrueOrBlankFieldType = 1244,
    LegPutOrCall: PutOrCallFieldType = 1358,
    EncryptedPasswordMethod: StringFieldType = 1400, //int
    EncryptedPasswordLen: NoneFieldType = 1401 => Rule::PrepareForBytes{ bytes_tag: EncryptedPassword::tag() },
    EncryptedPassword: DataFieldType = 1402 => Rule::ConfirmPreviousTag{ previous_tag: EncryptedPasswordLen::tag() },
    EncryptedNewPasswordLen: NoneFieldType = 1403 => Rule::PrepareForBytes{ bytes_tag: EncryptedNewPassword::tag() },
    EncryptedNewPassword: DataFieldType = 1404 => Rule::ConfirmPreviousTag{ previous_tag: EncryptedNewPasswordLen::tag() },
    RefApplExtID: StringFieldType = 1406, //int
    DefaultApplExtID: StringFieldType = 1407, //int
    DefaultCstmApplVerID: StringFieldType = 1408,
    SessionStatus: StringFieldType = 1409, //int
    DefaultVerIndicator: BoolTrueOrBlankFieldType = 1410,
    UnderlyingExerciseStyle: ExerciseStyleFieldType = 1419,
    LegExerciseStyle: ExerciseStyleFieldType = 1420,
    LegPriceUnitOfMeasure: UnitOfMeasureFieldType = 1421,
    LegPriceUnitOfMeasureQty: StringFieldType = 1422, //Qty
    UnderlyingUnitOfMeasureQty: StringFieldType = 1423, //Qty
    UnderlyingPriceUnitOfMeasure: UnitOfMeasureFieldType = 1424,
    UnderlyingPriceUnitOfMeasureQty: StringFieldType = 1425, //Qty
    ContractMultiplierUnit: ContractMultiplierUnitFieldType = 1435,
    LegContractMultiplierUnit: ContractMultiplierUnitFieldType = 1436,
    UnderlyingContractMultiplierUnit: ContractMultiplierUnitFieldType = 1437,
    FlowScheduleType: FlowScheduleTypeFieldType = 1439,
    LegFlowScheduleType: FlowScheduleTypeFieldType = 1440,
    UnderlyingFlowScheduleType: FlowScheduleTypeFieldType = 1441,
    NoRateSources: RepeatingGroupFieldType<RateSourceGrp> = 1445,
    RateSource: RateSourceFieldType = 1446,
    RateSourceType: RateSourceTypeFieldType = 1447,
    ReferencePage: StringFieldType = 1448,
    RestructuringType: RestructuringTypeFieldType = 1449,
    Seniority: SeniorityFieldType = 1450,
    NotionalPercentageOutstanding: PercentageFieldType = 1451,
    OriginalNotionalPercentageOutstanding: PercentageFieldType = 1452,
    UnderlyingRestructuringType: RestructuringTypeFieldType = 1453,
    UnderlyingSeniority: SeniorityFieldType = 1454,
    UnderlyingNotionalPercentageOutstanding: PercentageFieldType = 1455,
    UnderlyingOriginalNotionalPercentageOutstanding: PercentageFieldType = 1456,
    AttachmentPoint: PercentageFieldType = 1457,
    DetachmentPoint: PercentageFieldType = 1458,
    UnderlyingAttachmentPoint: PercentageFieldType = 1459,
    UnderlyingDetachmentPoint: PercentageFieldType = 1460,
    StrikePriceDeterminationMethod: StrikePriceDeterminationMethodFieldType = 1478,
    StrikePriceBoundaryMethod: StrikePriceBoundaryMethodFieldType = 1479,
    StrikePriceBoundaryPrecision: PercentageFieldType = 1480,
    UnderlyingPriceDeterminationMethod: UnderlyingPriceDeterminationMethodFieldType = 1481,
    OptPayoutType: OptPayoutTypeFieldType = 1482,
    NoComplexEvents: RepeatingGroupFieldType<ComplexEvent> = 1483,
    ComplexEventType: ComplexEventTypeFieldType = 1484,
    ComplexOptPayoutAmount: AmtFieldType = 1485,
    ComplexEventPrice: PriceFieldType = 1486,
    ComplexEventPriceBoundaryMethod: ComplexEventPriceBoundaryMethodFieldType = 1487,
    ComplexEventPriceBoundaryPrecision: PercentageFieldType = 1488,
    ComplexEventPriceTimeType: ComplexEventPriceTimeTypeFieldType = 1489,
    ComplexEventCondition: ComplexEventConditionFieldType = 1490,
    NoComplexEventDates: RepeatingGroupFieldType<ComplexEventDate> = 1491,
    ComplexEventStartDate: UTCTimestampFieldType = 1492, //TODO: Must always be less than end date.
    ComplexEventEndDate: UTCTimestampFieldType = 1493, //TODO: Must always be greater than event start date.
    NoComplexEventTimes: RepeatingGroupFieldType<ComplexEventTime> = 1494,
    ComplexEventStartTime: UTCTimeOnlyFieldType = 1495, //TODO: Must always be less than end time.
    ComplexEventEndTime: UTCTimeOnlyFieldType = 1496, //TODO: Must always be greater than start time.
);

//Repeating Groups (Sorted Alphabetically)

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

// define_message!(ComplexEventDate {
//     REQUIRED, complex_event_start_date: ComplexEventStartDate [FIX50SP2..],
//     REQUIRED, complex_event_end_date: ComplexEventEndDate [FIX50SP2..],
//     NOT_REQUIRED, no_complex_event_times: NoComplexEventTimes [FIX50SP2..],
// });

// define_message!(ComplexEventTime {
//     REQUIRED, complex_event_start_time: ComplexEventStartTime [FIX50SP2..],
//     REQUIRED, complex_event_end_time: ComplexEventEndTime [FIX50SP2..],
// });

// define_message!(EvntGrp {
//     REQUIRED, event_type: EventType [FIX44..],
//     NOT_REQUIRED, event_date: EventDate [FIX44..],
//     NOT_REQUIRED, event_time: EventTime [FIX50SP1..],
//     NOT_REQUIRED, event_px: EventPx [FIX44..],
//     NOT_REQUIRED, event_text: EventText [FIX44..],
// });

// define_message!(HopGrp {
//     REQUIRED, hop_comp_id: HopCompID [FIX43..],
//     NOT_REQUIRED, hop_sending_time: HopSendingTime [FIX43..],
//     NOT_REQUIRED, hop_ref_id: HopRefID [FIX43..],
// });

// define_message!(Instrument {
//     REQUIRED, related_sym: RelatedSym [FIX42],
//     REQUIRED, symbol: Symbol [FIX43..],
//     NOT_REQUIRED, symbol_sfx: SymbolSfx [FIX41..],
//     NOT_REQUIRED, security_id: SecurityID [FIX41..],
//     NOT_REQUIRED, security_id_source: SecurityIDSource [FIX41..] => REQUIRED_WHEN |message: &Instrument,_| { !message.security_id.is_empty() },
//     NOT_REQUIRED, no_security_alt_id: NoSecurityAltID [FIX43..],
//     NOT_REQUIRED, product: Product [FIX43..],
//     NOT_REQUIRED, product_complex: ProductComplex [FIX50SP1..],
//     NOT_REQUIRED, security_group: SecurityGroup [FIX50SP1..],
//     NOT_REQUIRED, cfi_code: CFICode [FIX43..],
//     NOT_REQUIRED, security_type: SecurityType [FIX41..] => REQUIRED_WHEN |message: &Instrument,_| { !message.security_sub_type.is_empty() },
//     NOT_REQUIRED, security_sub_type: SecuritySubType [FIX44..],
//     NOT_REQUIRED, maturity_month_year: MaturityMonthYear [FIX41..],
//     NOT_REQUIRED, maturity_month_day: MaturityDay [FIX41..FIX42],
//     NOT_REQUIRED, maturity_date: MaturityDate [FIX43..],
//     NOT_REQUIRED, maturity_time: MaturityTime [FIX50..],
//     NOT_REQUIRED, settle_on_open_flag: SettleOnOpenFlag [FIX50..],
//     NOT_REQUIRED, instrmt_assignment_method: InstrmtAssignmentMethod [FIX50..],
//     NOT_REQUIRED, security_status: SecurityStatus [FIX50..],
//     NOT_REQUIRED, coupon_payment_date: CouponPaymentDate [FIX43..],
//     NOT_REQUIRED, restructuring_type: RestructuringType [FIX50SP2..],
//     NOT_REQUIRED, seniority: Seniority [FIX50SP2..],
//     NOT_REQUIRED, notional_percentage_outstanding: NotionalPercentageOutstanding [FIX50SP2..],
//     NOT_REQUIRED, original_notional_percentage_outstanding: OriginalNotionalPercentageOutstanding [FIX50SP2..],
//     NOT_REQUIRED, attachment_point: AttachmentPoint [FIX50SP2..],
//     NOT_REQUIRED, detachment_point: DetachmentPoint [FIX50SP2..],
//     NOT_REQUIRED, issue_date: IssueDate [FIX43..],
//     NOT_REQUIRED, repo_collateral_security_type: RepoCollateralSecurityType [FIX43..],
//     NOT_REQUIRED, repurchase_term: RepurchaseTerm [FIX43..],
//     NOT_REQUIRED, repurchase_rate: RepurchaseRate [FIX43..],
//     NOT_REQUIRED, factor: Factor [FIX43..],
//     NOT_REQUIRED, credit_rating: CreditRating [FIX43..],
//     NOT_REQUIRED, instr_registry: InstrRegistry [FIX43..],
//     NOT_REQUIRED, country_of_issue: CountryOfIssue [FIX43..],
//     NOT_REQUIRED, state_or_province_of_issue: StateOrProvinceOfIssue [FIX43..],
//     NOT_REQUIRED, locale_of_issue: LocaleOfIssue [FIX43..],
//     NOT_REQUIRED, redemption_date: RedemptionDate [FIX43..],
//     NOT_REQUIRED, strike_price: StrikePrice [FIX41..],
//     NOT_REQUIRED, strike_currency: StrikeCurrency [FIX44..],
//     NOT_REQUIRED, strike_multiplier: StrikeMultiplier [FIX50..],
//     NOT_REQUIRED, strike_value: StrikeValue [FIX50..],
//     NOT_REQUIRED, strike_price_determination_method: StrikePriceDeterminationMethod [FIX50SP2..],
//     NOT_REQUIRED, strike_price_boundary_method: StrikePriceBoundaryMethod [FIX50SP2..],
//     NOT_REQUIRED, strike_price_boundary_precision: StrikePriceBoundaryPrecision [FIX50SP2..],
//     NOT_REQUIRED, underlying_price_determination_method: UnderlyingPriceDeterminationMethod [FIX50SP2..],
//     NOT_REQUIRED, opt_attribute: OptAttribute [FIX41..],
//     NOT_REQUIRED, contract_multiplier: ContractMultiplier [FIX42..],
//     NOT_REQUIRED, contract_multiplier_unit: ContractMultiplierUnit [FIX50SP2..],
//     NOT_REQUIRED, flow_schedule_type: FlowScheduleType [FIX50SP2..],
//     NOT_REQUIRED, min_price_increment: MinPriceIncrement [FIX50..],
//     NOT_REQUIRED, min_price_increment_amount: MinPriceIncrementAmount [FIX50SP1..],
//     NOT_REQUIRED, unit_of_measure: UnitOfMeasure [FIX50..],
//     NOT_REQUIRED, unit_of_measure_qty: UnitOfMeasureQty [FIX50SP1..],
//     NOT_REQUIRED, price_unit_of_measure: PriceUnitOfMeasure [FIX50SP1..],
//     NOT_REQUIRED, price_unit_of_measure_qty: PriceUnitOfMeasureQty [FIX50SP1..],
//     NOT_REQUIRED, settl_method: SettlMethod [FIX50SP1..],
//     NOT_REQUIRED, exercise_style: ExerciseStyle [FIX50SP1..],
//     NOT_REQUIRED, opt_payout_type: OptPayoutType [FIX50SP2..],
//     NOT_REQUIRED, opt_payout_amount: OptPayoutAmount [FIX50SP1..] => REQUIRED_WHEN |message: &Instrument,_| { if let Some(ref opt_payout_type) = message.opt_payout_type { *opt_payout_type == other_field_types::OptPayoutType::Binary } else { false } },
//     NOT_REQUIRED, price_quote_method: PriceQuoteMethod [FIX50SP1..],
//     NOT_REQUIRED, valuation_method: ValuationMethod [FIX50SP1..],
//     NOT_REQUIRED, list_method: ListMethod [FIX50SP1..],
//     NOT_REQUIRED, cap_price: CapPrice [FIX50SP1..],
//     NOT_REQUIRED, floor_price: FloorPrice [FIX50SP1..],
//     NOT_REQUIRED, put_or_call: PutOrCall [FIX41..],
//     NOT_REQUIRED, flexible_indicator: FlexibleIndicator [FIX50SP1..],
//     NOT_REQUIRED, flexible_product_eligibility_indicator: FlexibleProductElgibilityIndicator [FIX50SP1..],
//     NOT_REQUIRED, time_unit: TimeUnit [FIX50..],
//     NOT_REQUIRED, coupon_rate: CouponRate [FIX42..],
//     NOT_REQUIRED, security_exchange: SecurityExchange [FIX41..],
//     NOT_REQUIRED, position_limit: PositionLimit [FIX50..],
//     NOT_REQUIRED, nt_position_limit: NTPositionLimit [FIX50..],
//     NOT_REQUIRED, issuer: Issuer [FIX41..],
//     NOT_REQUIRED, encoded_issuer_len: EncodedIssuerLen [FIX42..],
//     NOT_REQUIRED, encoded_issuer: EncodedIssuer [FIX42..],
//     NOT_REQUIRED, security_desc: SecurityDesc [FIX41..],
//     NOT_REQUIRED, encoded_security_desc_len: EncodedSecurityDescLen [FIX42..],
//     NOT_REQUIRED, encoded_security_desc: EncodedSecurityDesc [FIX42..],
//     NOT_REQUIRED, security_xml_len: SecurityXMLLen [FIX50SP1..],
//     NOT_REQUIRED, security_xml: SecurityXML [FIX50SP1..],
//     NOT_REQUIRED, security_xml_schema: SecurityXMLSchema [FIX50SP1..],
//     NOT_REQUIRED, pool: Pool [FIX44..],
//     NOT_REQUIRED, contract_settl_month: ContractSettlMonth [FIX44..],
//     NOT_REQUIRED, cp_program: CPProgram [FIX44..],
//     NOT_REQUIRED, cp_reg_type: CPRegType [FIX44..],
//     NOT_REQUIRED, no_events: NoEvents [FIX44..],
//     NOT_REQUIRED, dated_date: DatedDate [FIX44..],
//     NOT_REQUIRED, interest_accrual_date: InterestAccrualDate [FIX44..],
//     NOT_REQUIRED, no_instrument_parties: NoInstrumentParties [FIX50..],
//     NOT_REQUIRED, no_complex_events: NoComplexEvents [FIX50SP2..],
// });

// define_message!(InstrumentLeg {
//     REQUIRED, leg_symbol: LegSymbol [FIX44..],
//     NOT_REQUIRED, leg_symbol_sfx: LegSymbolSfx [FIX44..],
//     NOT_REQUIRED, leg_security_id: LegSecurityID [FIX44..],
//     NOT_REQUIRED, leg_security_id_source: LegSecurityIDSource [FIX44..] => REQUIRED_WHEN |message: &InstrumentLeg,_| { !message.leg_security_id.is_empty() },
//     NOT_REQUIRED, no_leg_security_alt_id: NoLegSecurityAltID [FIX44..],
//     NOT_REQUIRED, leg_product: LegProduct [FIX44..],
//     NOT_REQUIRED, leg_cfi_code: LegCFICode [FIX44..],
//     NOT_REQUIRED, leg_security_type: LegSecurityType [FIX44..] => REQUIRED_WHEN |message: &InstrumentLeg,_| { !message.leg_security_sub_type.is_empty() },
//     NOT_REQUIRED, leg_security_sub_type: LegSecuritySubType [FIX44..],
//     NOT_REQUIRED, leg_maturity_month_year: LegMaturityMonthYear [FIX44..],
//     NOT_REQUIRED, leg_maturity_date: LegMaturityDate [FIX44..],
//     NOT_REQUIRED, leg_maturity_time: LegMaturityTime [FIX50SP1..],
//     NOT_REQUIRED, leg_coupon_payment_date: LegCouponPaymentDate [FIX44..],
//     NOT_REQUIRED, leg_issue_date: LegIssueDate [FIX44..],
//     NOT_REQUIRED, leg_repo_collateral_security_type: LegRepoCollateralSecurityType [FIX44..],
//     NOT_REQUIRED, leg_repurchase_term: LegRepurchaseTerm [FIX44..],
//     NOT_REQUIRED, leg_repurchase_rate: LegRepurchaseRate [FIX44..],
//     NOT_REQUIRED, leg_factor: LegFactor [FIX44..],
//     NOT_REQUIRED, leg_credit_rating: LegCreditRating [FIX44..],
//     NOT_REQUIRED, leg_instr_registry: LegInstrRegistry [FIX44..],
//     NOT_REQUIRED, leg_country_of_issue: LegCountryOfIssue [FIX44..],
//     NOT_REQUIRED, leg_state_or_province_of_issue: LegStateOrProvinceOfIssue [FIX44..],
//     NOT_REQUIRED, leg_locale_of_issue: LegLocaleOfIssue [FIX44..],
//     NOT_REQUIRED, leg_redemption_date: LegRedemptionDate [FIX44..],
//     NOT_REQUIRED, leg_strike_price: LegStrikePrice [FIX44..],
//     NOT_REQUIRED, leg_strike_currency: LegStrikeCurrency [FIX44..],
//     NOT_REQUIRED, leg_opt_attribute: LegOptAttribute [FIX44..],
//     NOT_REQUIRED, leg_contract_multiplier: LegContractMultiplier [FIX44..],
//     NOT_REQUIRED, leg_contract_multiplier_unit: LegContractMultiplierUnit [FIX50SP2..],
//     NOT_REQUIRED, leg_flow_schedule_type: LegFlowScheduleType [FIX50SP2..],
//     NOT_REQUIRED, leg_unit_of_measure: LegUnitOfMeasure [FIX50..],
//     NOT_REQUIRED, leg_unit_of_measure_qty: LegUnitOfMeasureQty [FIX50SP1..],
//     NOT_REQUIRED, leg_price_unit_of_measure: LegPriceUnitOfMeasure [FIX50SP1..],
//     NOT_REQUIRED, leg_price_unit_of_measure_qty: LegPriceUnitOfMeasureQty [FIX50SP1..],
//     NOT_REQUIRED, leg_time_unit: LegTimeUnit [FIX50..],
//     NOT_REQUIRED, leg_exercise_style: LegExerciseStyle [FIX50SP1..],
//     NOT_REQUIRED, leg_coupon_rate: LegCouponRate [FIX44..],
//     NOT_REQUIRED, leg_security_exchange: LegSecurityExchange [FIX44..],
//     NOT_REQUIRED, leg_issuer: LegIssuer [FIX44..],
//     NOT_REQUIRED, encoded_leg_issuer_len: EncodedLegIssuerLen [FIX44..],
//     NOT_REQUIRED, encoded_leg_issuer: EncodedLegIssuer [FIX44..],
//     NOT_REQUIRED, leg_security_desc: LegSecurityDesc [FIX44..],
//     NOT_REQUIRED, encoded_leg_security_desc_len: EncodedLegSecurityDescLen [FIX44..],
//     NOT_REQUIRED, encoded_leg_security_desc: EncodedLegSecurityDesc [FIX44..],
//     NOT_REQUIRED, leg_ratio_qty: LegRatioQty [FIX44..],
//     NOT_REQUIRED, leg_side: LegSide [FIX44..],
//     NOT_REQUIRED, leg_currency: LegCurrency [FIX44..],
//     NOT_REQUIRED, leg_poll: LegPool [FIX50..],
//     NOT_REQUIRED, leg_dated_date: LegDatedDate [FIX44..],
//     NOT_REQUIRED, leg_contract_settl_month: LegContractSettlMonth [FIX44..],
//     NOT_REQUIRED, leg_interest_accrual_date: LegInterestAccrualDate [FIX44..],
//     NOT_REQUIRED, leg_put_or_call: LegPutOrCall [FIX50SP1..],
//     NOT_REQUIRED, leg_option_ratio: LegOptionRatio [FIX50SP1..],
//     NOT_REQUIRED, leg_price: LegPrice [FIX50SP1..],
// });

// define_message!(InstrumentParty {
//     REQUIRED, instrument_party_id: InstrumentPartyID [FIX50..],
//     REQUIRED, instrument_party_id_source: InstrumentPartyIDSource [FIX50..], //Conditionally required if InstrumentPartyID is specified, but InstrumentPartyID is required, so this is also required.
//     NOT_REQUIRED, instrument_party_role: InstrumentPartyRole [FIX50..],
//     NOT_REQUIRED, no_instrument_party_sub_ids: NoInstrumentPartySubIDs [FIX50..],
// });

// define_message!(InstrumentPtysSubGrp {
//     REQUIRED, instrument_party_sub_id: InstrumentPartySubID [FIX50..],
//     REQUIRED, instrument_party_sub_id_type: InstrumentPartySubIDType [FIX50..],
// });

// define_message!(LegSecAltIDGrp {
//     REQUIRED, leg_security_alt_id: LegSecurityAltID [FIX44..],
//     REQUIRED, leg_security_alt_id_source: LegSecurityAltIDSource [FIX44..],
// });

// define_message!(LinesOfTextGrp {
//     REQUIRED, text: Text [FIX40..],
//     NOT_REQUIRED, encoded_text_len: EncodedTextLen [FIX42..],
//     NOT_REQUIRED, encoded_text: EncodedText [FIX42..],
// });

// define_message!(MsgTypeGrp {
//     REQUIRED, ref_msg_type: RefMsgType [FIX42..],
//     REQUIRED, msg_direction: MsgDirection [FIX42..],
//     NOT_REQUIRED, ref_appl_ver_id: RefApplVerID [FIX50..],
//     NOT_REQUIRED, ref_appl_ext_id: RefApplExtID [FIX50..],
//     NOT_REQUIRED, ref_cstm_appl_ver_id: RefCstmApplVerID [FIX50..],
//     NOT_REQUIRED, default_ver_indicator: DefaultVerIndicator [FIX50SP1..],
// });

// define_message!(RateSourceGrp {
//     REQUIRED, rate_source: RateSource [FIX50SP2..],
//     REQUIRED, rate_source_type: RateSourceType [FIX50SP2..],
//     NOT_REQUIRED, reference_page: ReferencePage [FIX50SP2..] => REQUIRED_WHEN |message: &RateSourceGrp,_| { message.rate_source == other_field_types::RateSource::Other },
// });

// define_message!(RoutingGrp {
//     REQUIRED, routing_type: RoutingType [FIX42..],
//     REQUIRED, routing_id: RoutingID [FIX42..],
// });

// define_message!(SecAltIDGrp {
//     REQUIRED, security_alt_id: SecurityAltID [FIX43..],
//     REQUIRED, security_alt_id_source: SecurityAltIDSource [FIX43..],
// });

// define_message!(UnderlyingInstrument {
//     REQUIRED, underlying_symbol: UnderlyingSymbol [FIX43..],
//     NOT_REQUIRED, underlying_symbol_sfx: UnderlyingSymbolSfx [FIX43..],
//     NOT_REQUIRED, underlying_security_id: UnderlyingSecurityID [FIX43..],
//     NOT_REQUIRED, underlying_security_id_source: UnderlyingSecurityIDSource [FIX43..] => REQUIRED_WHEN |message: &UnderlyingInstrument,_| { !message.underlying_security_id.is_empty() },
//     NOT_REQUIRED, no_underlying_security_alt_id: NoUnderlyingSecurityAltID [FIX43..],
//     NOT_REQUIRED, underlying_product: UnderlyingProduct [FIX43..],
//     NOT_REQUIRED, underlying_cfi_code: UnderlyingCFICode [FIX43..],
//     NOT_REQUIRED, underlying_security_type: UnderlyingSecurityType [FIX43..] => REQUIRED_WHEN |message: &UnderlyingInstrument,_| { !message.underlying_security_sub_type.is_empty() },
//     NOT_REQUIRED, underlying_security_sub_type: UnderlyingSecuritySubType [FIX44..],
//     NOT_REQUIRED, underlying_maturity_month_year: UnderlyingMaturityMonthYear [FIX43..],
//     NOT_REQUIRED, underlying_maturity_date: UnderlyingMaturityDate [FIX43..],
//     NOT_REQUIRED, underlying_maturity_time: UnderlyingMaturityTime [FIX50SP1..],
//     NOT_REQUIRED, underlying_coupon_payment_date: UnderlyingCouponPaymentDate [FIX43..],
//     NOT_REQUIRED, underlying_restructuring_type: UnderlyingRestructuringType [FIX50SP2..],
//     NOT_REQUIRED, underlying_seniority: UnderlyingSeniority [FIX50SP2..],
//     NOT_REQUIRED, underlying_notional_percentage_outstanding: UnderlyingNotionalPercentageOutstanding [FIX50SP2..],
//     NOT_REQUIRED, underlying_original_notional_percentage_outstanding: UnderlyingOriginalNotionalPercentageOutstanding [FIX50SP2..],
//     NOT_REQUIRED, underlying_attachment_point: UnderlyingAttachmentPoint [FIX50SP2..],
//     NOT_REQUIRED, underlying_detachment_point: UnderlyingDetachmentPoint [FIX50SP2..],
//     NOT_REQUIRED, underlying_issue_date: UnderlyingIssueDate [FIX43..],
//     NOT_REQUIRED, underlying_repo_collateral_security_type: UnderlyingRepoCollateralSecurityType [FIX43..],
//     NOT_REQUIRED, underlying_repurchase_term: UnderlyingRepurchaseTerm [FIX43..],
//     NOT_REQUIRED, underlying_repurchase_rate: UnderlyingRepurchaseRate [FIX43..],
//     NOT_REQUIRED, underlying_factor: UnderlyingFactor [FIX43..],
//     NOT_REQUIRED, underlying_credit_rating: UnderlyingCreditRating [FIX43..],
//     NOT_REQUIRED, underlying_instr_registry: UnderlyingInstrRegistry [FIX43..],
//     NOT_REQUIRED, underlying_country_of_issue: UnderlyingCountryOfIssue [FIX43..],
//     NOT_REQUIRED, underlying_state_or_province_of_issue: UnderlyingStateOrProvinceOfIssue [FIX43..],
//     NOT_REQUIRED, underlying_locale_of_issue: UnderlyingLocaleOfIssue [FIX43..],
//     NOT_REQUIRED, underlying_redemption_date: UnderlyingRedemptionDate [FIX43..],
//     NOT_REQUIRED, underlying_strike_price: UnderlyingStrikePrice [FIX43..],
//     NOT_REQUIREDderlying_currency: UnderlyingCurrency [FIX44..],
//     NOT_REQUIRED, underlying_qty: UnderlyingQty [FIX44..],
//     NOT_REQUIRED, underlying_settlement_type: UnderlyingSettlementType [FIX50..],
//     NOT_REQUIRED, underlying_cash_amount: UnderlyingCashAmount [FIX50..],
//     NOT_REQUIRED, underlying_cash_type: UnderlyingCashType [FIX50..],
//     NOT_REQUIRED, underlying_px: UnderlyingPx [FIX44..],
//     NOT_REQUIRED, underlying_dirty_price: UnderlyingDirtyPrice [FIX44..],
//     NOT_REQUIRED, underlying_end_price: UnderlyingEndPrice [FIX44..],
//     NOT_REQUIRED, underlying_start_value: UnderlyingStartValue [FIX44..],
//     NOT_REQUIRED, underlying_current_value: UnderlyingCurrentValue [FIX44..],
//     NOT_REQUIRED, underlying_end_value: UnderlyingEndValue [FIX44..],
//     NOT_REQUIRED, no_underlying_stips: NoUnderlyingStips [FIX44..],
//     NOT_REQUIRED, underlying_adjusted_quantity: UnderlyingAdjustedQuantity [FIX50..],
//     NOT_REQUIRED, underlying_fx_rate: UnderlyingFXRate [FIX50..],
//     NOT_REQUIRED, underlying_fx_rate_calc: UnderlyingFXRateCalc [FIX50..],
//     NOT_REQUIRED, underlying_cap_value: UnderlyingCapValue [FIX50..],
//     NOT_REQUIRED, no_undly_instrument_parties: NoUndlyInstrumentParties [FIX50..],
//     NOT_REQUIRED, underlying_settl_method: UnderlyingSettlMethod [FIX50..],
//     NOT_REQUIRED, underlying_put_or_call: UnderlyingPutOrCall [FIX50SP1..],
// });

// define_message!(UndlyInstrumentPtysSubGrp {
//     REQUIRED, underlying_instrument_party_sub_id: UnderlyingInstrumentPartySubID [FIX50..],
//     REQUIRED, underlying_instrument_party_sub_id_type: UnderlyingInstrumentPartySubIDType [FIX50..],
// });

// define_message!(UnderlyingStipulation {
//     REQUIRED, underlying_stip_type: UnderlyingStipType [FIX44..],
//     REQUIRED, underlying_stip_value: UnderlyingStipValue [FIX44..],
// });

// define_message!(UndSecAltIDGrp {
//     REQUIRED, underlying_security_alt_id: UnderlyingSecurityAltID [FIX43..],
//     REQUIRED, underlying_security_alt_id_source: UnderlyingSecurityAltIDSource [FIX43..],
// });

// , underlying_contract_multiplier_unit: UnderlyingContractMultiplierUnit [FIX50SP2..],
//     NOT_REQUIRED, underlying_flow_schedule_type: UnderlyingFlowScheduleType [FIX50SP2..],
//     NOT_REQUIRED, underlying_unit_of_measure: UnderlyingUnitOfMeasure [FIX50..],
//     NOT_REQUIRED, underlying_unit_of_measure_qty: UnderlyingUnitOfMeasureQty [FIX50SP1..],
//     NOT_REQUIRED, underlying_price_unit_of_measure: UnderlyingPriceUnitOfMeasure [FIX50SP1..],
//     NOT_REQUIRED, underlying_price_unit_of_measure_qty: UnderlyingPriceUnitOfMeasureQty [FIX50SP1..],
//     NOT_REQUIRED, underlying_time_unit: UnderlyingTimeUnit [FIX50..],
//     NOT_REQUIRED, underlying_exercise_style: UnderlyingExerciseStyle [FIX50SP1..],
//     NOT_REQUIRED, underlying_coupon_rate: UnderlyingCouponRate [FIX43..],
//     NOT_REQUIRED, underlying_security_exchange: UnderlyingSecurityExchange [FIX43..],
//     NOT_REQUIRED, underlying_issuer: UnderlyingIssuer [FIX43..],
//     NOT_REQUIRED, encoded_underlying_issuer_len: EncodedUnderlyingIssuerLen [FIX43..],
//     NOT_REQUIRED, encoded_underlying_issuer: EncodedUnderlyingIssuer [FIX43..],
//     NOT_REQUIRED, underlying_security_desc: UnderlyingSecurityDesc [FIX43..],
//     NOT_REQUIRED, encoded_underlying_security_desc_len: EncodedUnderlyingSecurityDescLen [FIX43..],
//     NOT_REQUIRED, encoded_underlying_security_desc: EncodedUnderlyingSecurityDesc [FIX43..],
//     NOT_REQUIRED, underlying_cp_program: UnderlyingCPProgram [FIX44..],
//     NOT_REQUIRED, underlying_cp_reg_type: UnderlyingCPRegType [FIX44..],
//     NOT_REQUIRED, underlying_allocation_percent: UnderlyingAllocationPercent [FIX50..],
//     NOT_REQUIRED, underlying_currency: UnderlyingCurrency [FIX44..],
//     NOT_REQUIRED, underlying_qty: UnderlyingQty [FIX44..],
//     NOT_REQUIRED, underlying_settlement_type: UnderlyingSettlementType [FIX50..],
//     NOT_REQUIRED, underlying_cash_amount: UnderlyingCashAmount [FIX50..],
//     NOT_REQUIRED, underlying_cash_type: UnderlyingCashType [FIX50..],
//     NOT_REQUIRED, underlying_px: UnderlyingPx [FIX44..],
//     NOT_REQUIRED, underlying_dirty_price: UnderlyingDirtyPrice [FIX44..],
//     NOT_REQUIRED, underlying_end_price: UnderlyingEndPrice [FIX44..],
//     NOT_REQUIRED, underlying_start_value: UnderlyingStartValue [FIX44..],
//     NOT_REQUIRED, underlying_current_value: UnderlyingCurrentValue [FIX44..],
//     NOT_REQUIRED, underlying_end_value: UnderlyingEndValue [FIX44..],
//     NOT_REQUIRED, no_underlying_stips: NoUnderlyingStips [FIX44..],
//     NOT_REQUIRED, underlying_adjusted_quantity: UnderlyingAdjustedQuantity [FIX50..],
//     NOT_REQUIRED, underlying_fx_rate: UnderlyingFXRate [FIX50..],
//     NOT_REQUIRED, underlying_fx_rate_calc: UnderlyingFXRateCalc [FIX50..],
//     NOT_REQUIRED, underlying_cap_value: UnderlyingCapValue [FIX50..],
//     NOT_REQUIRED, no_undly_instrument_parties: NoUndlyInstrumentParties [FIX50..],
//     NOT_REQUIRED, underlying_settl_method: UnderlyingSettlMethod [FIX50..],
//     NOT_REQUIRED, underlying_put_or_call: UnderlyingPutOrCall [FIX50SP1..],
// });

// define_message!(UndlyInstrumentPtysSubGrp {
//     REQUIRED, underlying_instrument_party_sub_id: UnderlyingInstrumentPartySubID [FIX50..],
//     REQUIRED, underlying_instrument_party_sub_id_type: UnderlyingInstrumentPartySubIDType [FIX50..],
// });

// define_message!(UnderlyingStipulation {
//     REQUIRED, underlying_stip_type: UnderlyingStipType [FIX44..],
//     REQUIRED, underlying_stip_value: UnderlyingStipValue [FIX44..],
// });

// define_message!(UndSecAltIDGrp {
//     REQUIRED, underlying_security_alt_id: UnderlyingSecurityAltID [FIX43..],
//     REQUIRED, underlying_security_alt_id_source: UnderlyingSecurityAltIDSource [FIX43..],
// });

