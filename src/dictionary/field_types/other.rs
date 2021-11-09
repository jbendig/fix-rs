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

#![allow(non_camel_case_types)]

use std::io::Write;
use std::str::FromStr;

use crate::field_type::FieldType;
use crate::fix_version::FIXVersion;
use crate::message::SetValueError;
use crate::message_version::MessageVersion;

//Enumerated Fields (Sorted Alphabetically)

//TODO: Make this public if the following issue ever gets fixed:
//      https://github.com/rust-lang/rust/pull/31179
//      Until then, just use MessageVersion directly outside of this module.
type ApplVerID = MessageVersion;

pub struct ApplVerIDFieldType;

impl FieldType for ApplVerIDFieldType {
    type Type = Option<ApplVerID>;

    fn default_value() -> Self::Type {
        None
    }

    fn set_value(field: &mut Self::Type, bytes: &[u8]) -> Result<(), SetValueError> {
        if let Some(value) = ApplVerID::from_bytes(bytes) {
            *field = Some(value);
            return Ok(());
        }

        return Err(SetValueError::OutOfRange);
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_none()
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(
        field: &Self::Type,
        _fix_version: FIXVersion,
        _message_version: MessageVersion,
        buf: &mut Vec<u8>,
    ) -> usize {
        if let Some(field) = *field {
            let bytes_value = field.as_bytes();
            return buf.write(bytes_value).unwrap();
        }

        0
    }
}

define_enum_field_type!(
    FIELD BusinessRejectReason {
        Other => b"0",
        UnknownID => b"1",
        UnknownSecurity => b"2",
        UnsupportedMessageType => b"3",
        ApplicationNotAvailable => b"4",
        ConditionallyRequiredFieldMissing => b"5",
        NotAuthorized => b"6",
        DeliverToFirmNotAvailableAtThisTime => b"7",
        InvalidPriceIncrement => b"18",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_INT] BusinessRejectReasonFieldType
);

define_enum_field_type!(
    FIELD ComplexEventCondition {
        And => b"1",
        Or => b"2",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] ComplexEventConditionFieldType
);

define_enum_field_type!(
    FIELD ComplexEventPriceBoundaryMethod {
        LessThanComplexEventPrice => b"1",
        LessThanOrEqualToComplexEventPrice => b"2",
        EqualToComplexEventPrice => b"3",
        GreaterThanOrEqualToComplexEventPrice => b"4",
        GreaterThanComplexEventPrice => b"5",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] ComplexEventPriceBoundaryMethodFieldType
);

define_enum_field_type!(
    FIELD ComplexEventPriceTimeType {
        Expiration => b"1",
        Immediate => b"2",
        SpecifiedDateTime => b"3",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] ComplexEventPriceTimeTypeFieldType
);

define_enum_field_type!(
    FIELD ComplexEventType {
        Capped => b"1",
        Trigger => b"2",
        KnockInUp => b"3",
        KnockInDown => b"4",
        KnockOutUp => b"5",
        KnockOutDown => b"6",
        Underlying => b"7",
        ResetBarrier => b"8",
        RollingBarrier => b"9",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_INT] ComplexEventTypeFieldType
);

define_enum_field_type!(
    FIELD ContractMultiplierUnit {
        Shares => b"0",
        Hours => b"1",
        Days => b"2",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] ContractMultiplierUnitFieldType
);

define_enum_field_type!(
    FIELD CPProgram {
        _3A3 => 1,
        _42 => 2,
        Other => 99,
    } Reserved100Plus => WITH_MINIMUM 100,
    FIELD_TYPE [NOT_REQUIRED] CPProgramFieldType
);

pub struct DefaultApplVerIDFieldType;

impl FieldType for DefaultApplVerIDFieldType {
    type Type = ApplVerID;

    fn default_value() -> Self::Type {
        //Default to FIX.5.0 because that's the first version to support versioned messages.
        MessageVersion::FIX50
    }

    fn set_value(field: &mut Self::Type, bytes: &[u8]) -> Result<(), SetValueError> {
        if let Some(value) = ApplVerID::from_bytes(bytes) {
            *field = value;
            return Ok(());
        }

        return Err(SetValueError::OutOfRange);
    }

    fn is_empty(_field: &Self::Type) -> bool {
        false
    }

    fn len(_field: &Self::Type) -> usize {
        0
    }

    fn read(
        field: &Self::Type,
        _fix_version: FIXVersion,
        _message_version: MessageVersion,
        buf: &mut Vec<u8>,
    ) -> usize {
        let bytes_value = field.as_bytes();
        return buf.write(bytes_value).unwrap();
    }
}

define_enum_field_type!(
    FIELD EmailType {
        New => b"0",
        Reply => b"1",
        AdminReply => b"2",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_CHAR] EmailTypeFieldType
);

define_enum_field_type!(
    FIELD EncryptMethod {
        None => b"0", //Or Other
        PKCS => b"1", //(Proprietary)
        DES => b"2", //(ECB Mode)
        PKCS_DES => b"3", //(Proprietary)
        PGP_DES => b"4", //(Defunct)
        PGP_DES_MD5 => b"5",
        PEM_DES_MD5 => b"6",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_INT] EncryptMethodFieldType
);

define_enum_field_type!(
    FIELD EventType {
        Put => 1,
        Call => 2,
        Tender => 3,
        SinkingFundCall => 4,
        Activation => 5,
        Inactivation => 6,
        LastEligibleTradeDate => 7,
        SwapStartDate => 8,
        SwapEndDate => 9,
        SwapRollDate => 10,
        SwapNextStartDate => 11,
        SwapNextRollDate => 12,
        FirstDeliveryDate => 13,
        LastDeliveryDate => 14,
        InitialInventoryDueDate => 15,
        FinalInventoryDueDate => 16,
        FirstIntentDate => 17,
        LastIntentDate => 18,
        PositionRemovalDate => 19,
        Other => 99,
    } Reserved100Plus => WITH_MINIMUM 100,
    FIELD_TYPE [REQUIRED] EventTypeFieldType
);

define_enum_field_type!(
    FIELD ExerciseStyle {
        European => b"0",
        American => b"1",
        Bermuda => b"2",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] ExerciseStyleFieldType
);

define_enum_field_type!(
    FIELD FlowScheduleType {
        NERCEasternOffPeak => 0,
        NERCWesternOffPeak => 1,
        NERCCalendar => 2,
        NERCEasternPeak => 3,
        NERCWesternPeak => 4,
    } Reserved100Plus => WITH_MINIMUM 100,
    FIELD_TYPE [NOT_REQUIRED] FlowScheduleTypeFieldType
);

define_enum_field_type!(
    FIELD HandlInst {
        AutomatedExecutionOrderPrivateNoBrokerIntervention => b"1",
        AutomatedExecutionOrderPublicBrokerInterventionOK => b"2",
        ManualOrderBestExecution => b"3",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_CHAR] HandlInstFieldType
);

define_enum_field_type!(
    FIELD InstrmtAssignmentMethod {
        Random => b"R",
        ProRata => b"P",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_CHAR] InstrmtAssignmentMethodFieldType
);

define_enum_field_type!(
    FIELD Issuer {
        CouncilOfEurope => b"COE",
        DeutscheAusgleichsbank => b"DTA",
        EuropeanBankForReconstructionAndDevelopment => b"EBRD",
        EuropeanInvestmentBank => b"EIB",
        Hessen => b"HESLAN",
        KreditanstaltFuerWiederaufbau => b"KFW",
        LandwirtschaftlicheRentenbank => b"LANREN",
        NordRheinWestfalenNRW => b"NORWES",
        SachsenAnhalt => b"SACHAN",
        AustrianTreasuryBill => b"RATB",
        AustrianGovernmentBond => b"RAGB",
        AustrianBundesobligation => b"AOBL", //OBL
        AustrianBundesschatzscheine => b"RABSS",
        AustrianGovernmentInternationalBond => b"AUST",
        RAGBCouponStrip => b"RAGBS", //Austrian
        RAGBPrincipalStrip => b"RAGBR", //Austrian
        AustriaMediumTermBill => b"RAMTB",
        BelgianTreasuryBill => b"BGTB",
        BelgianGovernmentBond => b"BGB",
        BelgianGovernmentInternationalBond => b"BELG",
        BelgianStrip => b"OLOS",
        BelgianPrincipalStrip => b"OLOR",
        DanishTreasuryBill => b"DGTB",
        DanishGovernmentBond => b"DGB",
        DanishGovernmentInternationalBond => b"DENK",
        FinnishTreasuryBill => b"RFTB",
        FinnishGovernmentBond => b"RFGB",
        FinnishGovernmentInternationalBond => b"FINL",
        FinnishHousingBond => b"FNHF",
        FrenchFixedRateShortTermDiscountTreasuryBills => b"BTF", //BTF
        FrenchFixedRateTreasuryNotes => b"BTNS", //BTAN
        FrenchTreasuryBonds => b"FRTR", //OAT
        FrenchTreasuryBondsPrincipalSTRIPS => b"FRTRR", //OAT
        FrenchTreasuryBondsCouponSTRIPS => b"FRTRS", //OAT
        SocialSecurityDebtRepaymentFund => b"CADES", //French
        GermanTreasuryBill => b"BUBILL",
        GermanFederalTreasuryBill => b"DBSB", //DM Ccy
        GermanTwoYearNotes => b"BKO",
        GermanFinancingTreasuryNotes => b"FSDB", //DM Ccy
        GermanGovernmentBond => b"DBR",
        GermanGovernmentBondPrincipalSTRIPS => b"DBRR",
        GermanGovernmentBondCouponSTRIPS => b"DBRS",
        GermanFiveYearBonds => b"OBL",
        GermanUnityFundDBR => b"DBRUF", //S (only 2)
        GermanUnityFund => b"BKOUF", //BKO (None)
        GermanFederalPost => b"DBP", //BUNDESPOST
        GermanFederalRailroad => b"DBB", //BUNDESBAHN
        TreuhandAgencyBonds => b"THA",
        TreuhandAgencyObligations => b"TOBL", //All matured
        GermanRetributionFund => b"ENTFND", //Only 2 sinking funds
        EuropeanRecoveryProgramSpecialFunds => b"GERP", //German only 2)
        Bundeskassenscheine => b"BUNKASS", //1 matured
        HellenicRepublicTreasuryBill => b"GTB",
        HellenicRepublicGovernmentBond => b"GGB",
        HellenicRepublicGovernmentInternationalBond => b"GREECE",
        HellenicRepublicGovernmentBondCouponSTRIPS => b"GGBSTP",
        HellenicRepublicGovernmentBondResidualSTRIPS => b"GGBRES",
        IrishGovernmentBond => b"IRISH",
        IrishGovernmentInternationalBond => b"IRELND",
        ItalianTreasuryBill => b"BOTS",
        ItalianGovernmentBond => b"BTPS",
        ItalianTreasuryCertificate => b"CCTS",
        ItalianZeroCouponBonds => b"ICTZ",
        ItalianGovernmentBondsIssuedInEUR => b"CTES", //Matured
        ItalianGovernmentBondsWithPutOption => b"CTOS", //All matured
        ItalianInternationalBonds => b"ITALY",
        ItalianGovernmentBondCouponSTRIPS => b"BTPSS",
        ItalianGovernmentBondResidualSTRIPS => b"BTPSR",
        LuxembourgeoisGovernmentBond => b"LGB",
        DutchGovernmentBond => b"NETHER",
        DutchPrincipalStrip => b"NETHRR",
        DutchStrip => b"NETHRS",
        DutchTreasuryCertificate => b"DTB",
        DutchBankCertificate => b"NBC", //All matured
        NorwegianTreasuryBill => b"NGTB",
        NorwegianGovernmentBond => b"NGB",
        NorwegianGovernmentInternationalBond => b"NORWAY", //NOK
        PortugueseTreasuryBills => b"PORTB",
        PortugueseGovernmentBond => b"PGB",
        PortugueseGovernmentInternationalBond => b"PORTUG",
        SpanishGovernmentBond => b"SPGB",
        SpanishGovernmentBondCouponStrips => b"SPGBS",
        SpanishGovernmentBondPrincipalStrips => b"SPGBR",
        SpanishGovernmentInternationalBond => b"SPAIN",
        SpanishLetrasDelTesoro => b"SGLT",
        SwedishTreasuryBill => b"SWTB",
        SwedishGovernmentBond => b"SGB",
        SwedishGovernmentInternationalBond => b"SWED", //SEK
        SwedishGovernmentBondCouponStrip => b"SGBS",
        SwedishGovernmentBondResidualStrip => b"SGBR",
        SwissTreasuryBill => b"SWISTB",
        SwissGovernmentBond => b"SWISS",
        GenevaTreasuryBill => b"GENTB", //CHF
        UnitedKingdomGBPOrEURTreasuryBill => b"UKTB",
        UnitedKingdomGiltBond => b"UKT",
        UnitedKingdomGiltBondCouponSTRIPS => b"UKTS",
        UnitedKingdomGiltBondResidualSTRIPS => b"UKTR",
        UnitedKingdomInternationalBond => b"UKIN",
        BankOfEnglandEURBill => b"BOE",
        BankOfEnglandEURNote => b"BOEN",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_STRING] IssuerFieldType
);

define_enum_field_type!(
    FIELD ListMethod {
        PreListedOnly => b"0",
        UserRequested => b"1",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] ListMethodFieldType
);

define_enum_field_type!(
    FIELD MsgDirection {
        Receive => b"R",
        Send => b"S",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_CHAR] MsgDirectionFieldType
);

define_enum_field_type!(
    FIELD OptPayoutType {
        Vanilla => b"1",
        Capped => b"2",
        Binary => b"3",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] OptPayoutTypeFieldType
);

define_enum_field_type!(
    FIELD OrdType {
        Market => b"1",
        Limit => b"2",
        StopOrStopLoss => b"3",
        StopLimit => b"4",
        MarketOnClose => b"5", //Deprecated in FIX 4.3
        WithOrWithout => b"6",
        LimitOrBetter => b"7", //Deprecated in FIX 4.4
        LimitWithOrWithout => b"8",
        OnBasis => b"9",
        OnClose => b"A", //Deprecated in FIX 4.3
        LimitOnClose => b"B", //Deprecated in FIX 4.3
        ForexMarket => b"C", //Deprecated in FIX 4.3
        PreviouslyQuoted => b"D",
        PreviouslyIndicated => b"E",
        ForexLimit => b"F", //Deprecated in FIX 4.3
        ForexSwap => b"G",
        ForexPreviouslyQuoted => b"H", //Deprecated in FIX 4.3
        Funari => b"I",
        MarketIfTouched => b"J",
        MarketWithLeftOverAsLimit => b"K",
        PreviousFundValuationPoint => b"L",
        NextFundValuationPoint => b"M",
        Pegged => b"P",
        CounterOrderSelection => b"Q",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_CHAR] OrdTypeFieldType
);

define_enum_field_type!(
    FIELD PartyIDSource {
        BIC => b"B",
        GenerallyAcceptedMarketParticipantIdentifier => b"C",
        ProprietaryOrCustomCode => b"D",
        ISOCountryCode => b"E",
        SettlementEntityLocation => b"F",
        MIC => b"G",
        CSDParticipantOrMemberCode => b"H",
        UKNationalInsuranceOrPensionNumber => b"6",
        USSocialSecurityNumber => b"7",
        USEmployerOrTaxIDNumber => b"8",
        AustralianBusinessNumber => b"9",
        AustralianTaxFileNumber => b"A",
        KoreanInvestorID => b"1",
        TaiwaneseQualifiedForeignInvestorIDQFIIOrFID => b"2",
        TaiwaneseTradingAcct => b"3",
        MalaysianCentralDepositoryNumber => b"4",
        ChineseInvestorID => b"5",
        DirectedBroker => b"I",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_CHAR] PartyIDSourceFieldType
);

define_enum_field_type!(
    FIELD PartyRole {
        CentralRegistrationDepository => b"82",
        ClearingAccount => b"83",
        AcceptableSettlingCounterparty => b"84",
        UnacceptableSettlingCounterparty => b"85",
        ExecutingFirm => b"1",
        BrokerOfCredit => b"2",
        ClientID => b"3",
        ClearingFirm => b"4",
        InvestorID => b"5",
        IntroducingFirm => b"6",
        EnteringFirm => b"7",
        LocateOrLendingFirm => b"8",
        FundManagerClientID => b"9",
        SettlementLocation => b"10",
        OrderOriginationTrader => b"11",
        ExecutingTrader => b"12",
        OrderOriginationFirm => b"13",
        GiveupClearingFirm => b"14",
        CorrespondantClearingFirm => b"15",
        ExecutingSystem => b"16",
        ContraFirm => b"17",
        ContraClearingFirm => b"18",
        SponsoringFirm => b"19",
        UnderlyingContraFirm => b"20",
        ClearingOrganization => b"21",
        Exchange => b"22",
        CustomerAccount => b"24",
        CorrespondentClearingOrganization => b"25",
        CorrespondentBroker => b"26",
        BuyerOrSeller => b"27",
        Custodian => b"28",
        Intermediary => b"29",
        Agent => b"30",
        SubCustodian => b"31",
        Beneficiary => b"32",
        InterestedParty => b"33",
        RegulatoryBody => b"34",
        LiquidityProvider => b"35",
        EnteringTrader => b"36",
        ContraTrader => b"37",
        PositionAccount => b"38",
        ContraInvestorID => b"39",
        TransferToFirm => b"40",
        ContraPositionAccount => b"41",
        ContraExchange => b"42",
        InternalCarryAccount => b"43",
        OrderEntryOperatorID => b"44",
        SecondaryAccountNumber => b"45",
        ForeignFirm => b"46",
        ThirdPartyAllocationFirm => b"47",
        ClaimingAccount => b"48",
        AssetManager => b"49",
        PledgorAccount => b"50",
        PledgeeAccount => b"51",
        LargeTraderReportableAccount => b"52",
        TraderMnemonic => b"53",
        SenderLocation => b"54",
        SessionID => b"55",
        AcceptableCounterparty => b"56",
        UnacceptableCounterparty => b"57",
        EnteringUnit => b"58",
        ExecutingUnit => b"59",
        IntroducingBroker => b"60",
        QuoteOriginator => b"61",
        ReportOriginator => b"62",
        SystematicInternaliser => b"63",
        MultilateralTradingFacility => b"64",
        RegulatedMarket => b"65",
        MarketMaker => b"66",
        InvestmentFirm => b"67",
        HostCompetentAuthority => b"68",
        HomeCompetentAuthority => b"69",
        CompetentAuthorityOfTheMostRelevantMarketInTermsOfLiquidity => b"70",
        CompetentAuthorityOfTheTransactionVenue => b"71",
        ReportingIntermediary => b"72",
        ExecutionVenue => b"73",
        MarketDataEntryOriginator => b"74",
        LocationID => b"75",
        DeskID => b"76",
        MarketDataMarket => b"77",
        AllocationEntity => b"78",
        PrimeBrokerProvidingGeneralTradeServices => b"79",
        StepOutFirm => b"80",
        BrokerClearingID => b"81",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] PartyRoleFieldType
);

define_enum_field_type!(
    FIELD PartySubIDType {
        Firm => 1,
        Person => 2,
        System => 3,
        Application => 4,
        FullLegalNameOfFirm => 5,
        PostalAddress => 6,
        PhoneNumber => 7,
        EmailAddress => 8,
        ContactName => 9,
        SecuritiesAccountNumberForSettlementInstructions => 10,
        RegistrationNumberForSettlementInstructionsAndConfirmations => 11,
        RegisteredAddressForConfirmationPurposes => 12,
        RegulatoryStatusForConfirmationPurposes => 13,
        RegistrationNameForSettlementInstructions => 14,
        CashAccountNumberForSettlementInstructions => 15,
        BIC => 16,
        CSDParticipantMemberCode => 17,
        RegisteredAddress => 18,
        FundAccountName => 19,
        TelexNumber => 20,
        FaxNumber => 21,
        SecuritiesAccountName => 22,
        CashAccountName => 23,
        Department => 24,
        LocationDesk => 25,
        PositionAccountType => 26,
        SecurityLocateID => 27,
        MarketMaker => 28,
        ElgibleCounterparty => 29,
        ProfessionalClient => 30,
        Location => 31,
        ExecutionVenue => 32,
        CurrencyDeliveryIdentifier => 33,
    } Reserved4000Plus => WITH_MINIMUM 4000,
    FIELD_TYPE [REQUIRED] PartySubIDTypeFieldType
);

define_enum_field_type!(
    FIELD PriceQuoteMethod {
        PercentOfPar => b"PCTPAR",
        Standard => b"STD",
        Index => b"INDX",
        InterestRateIndex => b"INT",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_STRING] PriceQuoteMethodFieldType
);

define_enum_field_type!(
    FIELD Product {
        Agency => b"1",
        Commodity => b"2",
        Corporate => b"3",
        Currency => b"4",
        Equity => b"5",
        Government => b"6",
        Index => b"7",
        Loan => b"8",
        MoneyMarket => b"9",
        Mortgage => b"10",
        Municipal => b"11",
        Other => b"12",
        Financing => b"13",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] ProductFieldType
);

define_enum_field_type!(
    FIELD PutOrCall {
        Put => b"0",
        Call => b"1",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] PutOrCallFieldType
);

define_enum_field_type!(
    FIELD RateSource {
        Bloomberg => b"0",
        Reuters => b"1",
        Telerate => b"2",
        Other => b"99",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_INT] RateSourceFieldType
);

define_enum_field_type!(
    FIELD RateSourceType {
        Primary => b"0",
        Secondary => b"1",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_INT] RateSourceTypeFieldType
);

define_enum_field_type!(
    FIELD RestructuringType {
        FullRestructuring => b"FR",
        ModifiedRestructuring => b"MR",
        ModifiedModRestructuring => b"MM",
        NoRestructuringSpecified => b"XR",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_STRING] RestructuringTypeFieldType
);

define_enum_field_type!(
    FIELD RoutingType {
        TargetFirm => b"1",
        TargetList => b"2",
        BlockFirm => b"3",
        BlockList => b"4",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_INT] RoutingTypeFieldType
);

define_enum_field_type!(
    FIELD SecurityIDSource {
        CUSIP => b"1",
        SEDOL => b"2",
        QUIK => b"3",
        ISINNumber => b"4",
        RICCode => b"5",
        ISOCurrencyCode => b"6",
        ISOCountryCode => b"7",
        ExchangeSymbol => b"8",
        ConsolidatedTapeAssociationSymbol => b"9",
        BloombergSymbol => b"A",
        Wertpapier => b"B",
        Dutch => b"C",
        Valoren => b"D",
        Sicovam => b"E",
        Belgian => b"F",
        Common => b"G",
        ClearingHouseOrClearingOrganization => b"H",
        ISDAOrFpMLProductSpecification => b"I",
        OptionPriceReportingAuthority => b"J",
        ISDAOrFpMLProductURL => b"K",
        LetterOfCredit => b"L",
        MarketplaceAssignedIdentifier => b"M",
    } Other,
    FIELD_TYPE [REQUIRED_AND_NOT_REQUIRED,BYTES] RequiredSecurityIDSourceFieldType NotRequiredSecurityIDSourceFieldType
);

define_enum_field_type!(
    FIELD SecurityStatus {
        Active => b"1",
        Inactive => b"2",
    },
    FIELD_TYPE [REQUIRED,MUST_BE_STRING] SecurityStatusFieldType
);

define_enum_field_type!(
    FIELD SecurityType {
        USTreasureNote => b"UST", //Deprecated in FIX 4.4, use USTreasuryNote.
        USTreasureBill => b"USTB", //Deprecated in FIX 4.4, use USTreasuryBill.
        EuroSupranationalCoupons => b"EUSUPRA",
        FederalAgencyCoupon => b"FAC",
        FederalAgencyDiscountNote => b"FADN",
        PrivateExportFunding => b"PEF",
        USDSupranationalCoupons => b"SUPRA",
        CorporateBond => b"CORP",
        CorporatePrivatePlacement => b"CPP",
        ConvertibleBond => b"CB",
        DualCurrency => b"DUAL",
        EuroCorporateBond => b"EUCORP",
        EuroCorporateFloatingRateNotes => b"EUFRN",
        USCorporateFloatingRateNotes => b"FRN",
        IndexedLinked => b"XLINKD",
        StructuredNotes => b"STRUCT",
        YankeeCorporateBond => b"YANK",
        ForeignExchangeContract => b"FOR", //Deprecated in FIX 5.2 SP1
        NonDeliverableForward => b"FXNDF",
        FXSpot => b"FXSPOT",
        FXForward => b"FXFWD",
        FXSwap => b"FXSWAP",
        CreditDefaultSwap => b"CDS",
        Future => b"FUT",
        Option => b"OPT",
        OptionsOnFutures => b"OOF",
        OptionsOnPhysical => b"OOP",
        InterestRateSwap => b"IRS",
        OptionsOnCombo => b"OOC",
        CommonStock => b"CS",
        PreferredStock => b"PS",
        Repurchase => b"REPO",
        Forward => b"FORWARD",
        BuySellback => b"BUYSELL",
        SecuritiesLoan => b"SECLOAN",
        SecuritiesPledge => b"SECPLEDGE",
        BradyBond => b"BRADY",
        CanadianTreasuryNotes => b"CAN",
        CanadianTreasuryBills => b"CTB",
        EuroSovereigns => b"EUSOV",
        CanadianProvincialBonds => b"PROV",
        TreasuryBillNonUS => b"TB",
        USTreasuryBond => b"TBOND",
        InterestStripFromAnyBondOrNote => b"TINT",
        USTreasuryBill => b"TBILL",
        TreasuryInflationProtectedSecurities => b"TIPS",
        PrincipalStripOfACallableBondOrNote => b"TCAL",
        PrincipalStripFromANonCallableBondOrNote => b"TPRN",
        USTreasuryNote => b"TNOTE",
        TermLoan => b"TERM",
        RevolverLoan => b"RVLV",
        RevolverOrTermLoan => b"RVLTRM",
        BridgeLoan => b"BRIDGE",
        LetterOfCredit => b"LOFC",
        SwingLineFacility => b"SWING",
        DebtorInPossession => b"DINP",
        Defaulted => b"DEFLTED",
        Withdrawn => b"WITHDRN",
        Replaced => b"REPLACD",
        Matured => b"MATURED",
        AmendedAndRestated => b"AMENDED",
        Retired => b"RETIRED",
        BankersAcceptance => b"BA",
        BankDepositoryNote => b"BDN",
        BankNotes => b"BN",
        BillOfExchanges => b"BOX",
        CanadianMoneyMarkets => b"CAMM",
        CertificateOfDeposit => b"CD",
        CallLoans => b"CL",
        CommercialPaper => b"CP",
        DepositNotes => b"DN",
        EuroCertificateOfDeposit => b"EUCD",
        EuroCommercialPaper => b"EUCP",
        LiquidityNote => b"LQN",
        MediumTermNotes => b"MTN",
        Overnight => b"ONITE",
        PromissoryNote => b"PN",
        ShortTermLoanNote => b"STN",
        PlazosFijos => b"PZFJ",
        SecuredLiquidityNote => b"SLQN",
        TimeDeposit => b"TD",
        TermLiquidityNote => b"TLQN",
        ExtendedCommNote => b"XCN",
        YankeeCertificateOfDeposit => b"YCD",
        AssetBackedSecurities => b"ABS",
        CanadianMortgageBonds => b"CMB",
        CorpMortgageBackedSecurities => b"CMBS",
        CollateralizedMortgageObligation => b"CMO",
        IOETTEMortgage => b"IET",
        MortgageBackedSecurities => b"MBS",
        MortgageInterestOnly => b"MIO",
        MortgagePrincipalOnly => b"MPO",
        MortgagePrivatePlacement => b"MPP",
        MiscellaneousPassThrough => b"MPT",
        Pfandbriefe => b"PFAND",
        ToBeAnnounced => b"TBA",
        OtherAnticipationNotes => b"AN",
        CertificateOfObligation => b"COFO",
        CertificateOfParticipation => b"COFP",
        GeneralObligationBonds => b"GO",
        MandatoryTender => b"MT",
        RevenueAnticipationNote => b"RAN",
        RevenueBonds => b"REV",
        SpecialAssessment => b"SPCLA",
        SpecialObligation => b"SPCLO",
        SpecialTax => b"SPCLT",
        TaxAnticipationNote => b"TAN",
        TaxAllocation => b"TAXA",
        TaxExemptCommercialPaper => b"TECP",
        TaxableMunicipalCP => b"TMCP",
        TaxRevenueAnticipationNote => b"TRAN",
        VariableRateDemandNote => b"VRDN",
        Warrant => b"WAR",
        MutualFund => b"MF",
        MultilegInstrument => b"MLEG",
        NoSecurityType => b"NONE",
        Wildcard => b"?",
        Cash => b"CASH",
    } Other,
    FIELD_TYPE [REQUIRED_AND_NOT_REQUIRED,BYTES] RequiredSecurityTypeFieldType NotRequiredSecurityTypeFieldType
);

define_enum_field_type!(
    FIELD Seniority {
        SeniorSecured => b"SD",
        Senior => b"SR",
        Subordinated => b"SB",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_STRING] SeniorityFieldType
);

define_enum_field_type!(
    FIELD SessionRejectReason {
        InvalidTagNumber => 0,
        RequiredTagMissing => 1,
        TagNotDefinedForThisMessageType => 2,
        UndefinedTag => 3,
        TagSpecifiedWithoutAValue => 4,
        ValueIsIncorrectForThisTag => 5,
        IncorrectDataFormatForValue => 6,
        DecryptionProblem => 7,
        SignatureProblem => 8,
        CompIDProblem => 9,
        SendingTimeAccuracyProblem => 10,
        InvalidMsgType => 11,
        XMLValidationError => 12,
        TagAppearsMoreThanOnce => 13,
        TagSpecifiedOutOfRequiredOrder => 14,
        RepeatingGroupFieldsOutOfOrder => 15,
        IncorrectNumInGroupCountForRepeatingGroup => 16,
        NonDataValueIncludesFieldDelimiter => 17,
        InvalidOrUnsupportedApplicationVersion => 18,
        Other => 99,
    } Reserved100Plus => WITH_MINIMUM 100,
    FIELD_TYPE [NOT_REQUIRED] SessionRejectReasonFieldType
);

define_enum_field_type!(
    FIELD SettlMethod {
        CashSettlementRequired => b"C",
        PhysicalSettlementRequired => b"P",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_CHAR] SettlMethodFieldType
);

#[derive(Clone, PartialEq)]
pub enum SettlType {
    RegularOrFXSpotSettlement,
    Cash,
    NextDay,
    TPlus2,
    TPlus3,
    TPlus4,
    Future,
    WhenAndIfIssued,
    SellersOption,
    TPlus5,
    BrokenDate,
    FXSpotNextSettlement,
    Days(u64),
    Months(u64),
    Weeks(u64),
    Years(u64),
}

impl SettlType {
    fn new(bytes: &[u8]) -> Option<SettlType> {
        Some(match bytes {
            b"0" => SettlType::RegularOrFXSpotSettlement,
            b"1" => SettlType::Cash,
            b"2" => SettlType::NextDay,
            b"3" => SettlType::TPlus2,
            b"4" => SettlType::TPlus3,
            b"5" => SettlType::TPlus4,
            b"6" => SettlType::Future,
            b"7" => SettlType::WhenAndIfIssued,
            b"8" => SettlType::SellersOption,
            b"9" => SettlType::TPlus5,
            b"B" => SettlType::BrokenDate,
            b"C" => SettlType::FXSpotNextSettlement,
            _ if bytes.len() > 1 => {
                let number_string = String::from_utf8_lossy(&bytes[1..]).into_owned();
                let number = match u64::from_str(&number_string) {
                    Err(_) => return None,
                    Ok(number) if number == 0 => {
                        //All of the following tenors require an integer > 0.
                        return None;
                    }
                    Ok(number) => number,
                };

                match bytes[0] {
                    b'D' => SettlType::Days(number),
                    b'M' => SettlType::Months(number),
                    b'W' => SettlType::Weeks(number),
                    b'Y' => SettlType::Years(number),
                    _ => return None,
                }
            }
            _ => return None,
        })
    }

    fn read(&self, buf: &mut Vec<u8>) -> usize {
        match *self {
            SettlType::RegularOrFXSpotSettlement => buf.write(b"0").unwrap(),
            SettlType::Cash => buf.write(b"1").unwrap(),
            SettlType::NextDay => buf.write(b"2").unwrap(),
            SettlType::TPlus2 => buf.write(b"3").unwrap(),
            SettlType::TPlus3 => buf.write(b"4").unwrap(),
            SettlType::TPlus4 => buf.write(b"5").unwrap(),
            SettlType::Future => buf.write(b"6").unwrap(),
            SettlType::WhenAndIfIssued => buf.write(b"7").unwrap(),
            SettlType::SellersOption => buf.write(b"8").unwrap(),
            SettlType::TPlus5 => buf.write(b"9").unwrap(),
            SettlType::BrokenDate => buf.write(b"B").unwrap(),
            SettlType::FXSpotNextSettlement => buf.write(b"C").unwrap(),
            SettlType::Days(number) => {
                buf.write(b"D").unwrap() + buf.write(number.to_string().as_bytes()).unwrap()
            }
            SettlType::Months(number) => {
                buf.write(b"M").unwrap() + buf.write(number.to_string().as_bytes()).unwrap()
            }
            SettlType::Weeks(number) => {
                buf.write(b"W").unwrap() + buf.write(number.to_string().as_bytes()).unwrap()
            }
            SettlType::Years(number) => {
                buf.write(b"Y").unwrap() + buf.write(number.to_string().as_bytes()).unwrap()
            }
        }
    }
}

pub struct SettlTypeFieldType;

impl FieldType for SettlTypeFieldType {
    type Type = Option<SettlType>;

    fn default_value() -> Self::Type {
        None
    }

    fn set_value(field: &mut Self::Type, bytes: &[u8]) -> Result<(), SetValueError> {
        if let Some(value) = SettlType::new(bytes) {
            *field = Some(value);
            return Ok(());
        }

        Err(SetValueError::OutOfRange)
    }

    fn is_empty(field: &Self::Type) -> bool {
        field.is_none()
    }

    fn len(_field: &Self::Type) -> usize {
        0 //Unused for this type.
    }

    fn read(
        field: &Self::Type,
        _fix_version: FIXVersion,
        _message_version: MessageVersion,
        buf: &mut Vec<u8>,
    ) -> usize {
        if let Some(ref field) = *field {
            return field.read(buf);
        }

        0
    }
}

define_enum_field_type!(
    FIELD Side {
        Buy => b"1",
        Sell => b"2",
        BuyMinus => b"3",
        SellPlus => b"4",
        SellShort => b"5",
        SellShortExempt => b"6",
        Undisclosed => b"7",
        Cross => b"8",
        CrossShort => b"9",
        CrossShortExempt => b"A",
        AsDefined => b"B",
        Opposite => b"C",
        Subscribe => b"D",
        Redeem => b"E",
        Lend => b"F",
        Borrow => b"G",
    },
    FIELD_TYPE [REQUIRED_AND_NOT_REQUIRED,MUST_BE_CHAR] RequiredSideFieldType NotRequiredSideFieldType
);

define_enum_field_type!(
    FIELD StipulationType {
        AlternativeMinimumTax => b"AMT",
        AutoReinvestment => b"AUTOREINV",
        BankQualified => b"BANKQUAL",
        BargainConditions => b"BGNCON",
        CouponRange => b"COUPON",
        ISOCurrencyCode => b"CURRENCY",
        CustomStartOrEndDate => b"CUSTOMDATE",
        Geographics => b"GEOG",
        ValuationDiscount => b"HAIRCUT",
        Insured => b"ISSURED",
        YearOrYearAndMonthOfIssue => b"ISSUE",
        IssuersTicker => b"ISSUER",
        IssueSizeRange => b"ISSUESIZE",
        LookbackDays => b"LOOKBACK",
        ExplicitLotIdentifier => b"LOT",
        LotVariance => b"LOTVAR",
        MaturityYearAndMonth => b"MAT",
        MaturityRange => b"MATURITY",
        MaximumSubstitutions => b"MAXSUBS",
        MinimumDenomination => b"MINDNOM",
        MinimumIncrement => b"MININCR",
        MinimumQuantity => b"MINQTY",
        PaymentFrequency => b"PAYFREQ",
        NumberOfPieces => b"PIECES",
        PoolsMaximum => b"PMAX",
        PoolsPerLot => b"PPL",
        PoolsPerMillion => b"PPM",
        PoolsPerTrade => b"PPT",
        PriceRange => b"PRICE",
        PricingFrequency => b"PRICEFREQ",
        ProductionYear => b"PROD",
        CallProtection => b"PROTECT",
        Purpose => b"PURPOSE",
        BenchmarkPriceSource => b"PXSOURCE",
        RatingSourceAndRange => b"RATING",
        TypeOfRedemption => b"REDEMPTION",
        Restricted => b"RESTRICTED",
        MarketSector => b"SECTOR",
        SecurityType => b"SECTYPE",
        Structure => b"STRUCT",
        SubstitutionsFrequency => b"SUBSFREQ",
        SubstitutionsLeft => b"SUBSLEFT",
        FreeformText => b"TEXT",
        TradeVariance => b"TRDVAR",
        WeightedAverageCoupon => b"WAC",
        WeightedAverageLifeCoupon => b"WAL",
        WeightedAverageLoanAge => b"WALA",
        WeightedAverageMaturity => b"WAM",
        WholePool => b"WHOLE",
        YieldRange => b"YIELD",
        AverageFICOScore => b"AVFICO",
        AverageLoanSize => b"AVSIZE",
        MaximumLoanBalance => b"MAXBAL",
        PoolIdentifier => b"POOL",
        TypeOfRollTrade => b"ROLLTYPE",
        ReferenceToRollingOrClosingTrade => b"REFTRADE",
        PrincipalOfRollingOrClosingTrade => b"REFPRIN",
        InterestOfRollingOrClosingTrade => b"REFINT",
        AvailableOfferQuantityToBeShownToTheStreet => b"AVAILQTY",
        BrokersSalesCredit => b"BROKERCREDIT",
        OfferPriceToBeShownToInternalBrokers => b"INTERNALPX",
        OfferQuantityToBeShownToInternalBrokers => b"INTERNALQTY",
        MinimumResidualOfferQuantity => b"LEAVEQTY",
        MaximumOrderSize => b"MAXORDQTY",
        OrderQuantityIncrement => b"ORDRINCR",
        PrimaryOrSecondaryMarketIndicator => b"PRIMARY",
        BrokerSalesCreditOverride => b"SALESCREDITOVR",
        TradersCredit => b"TRADERCREDIT",
        DiscountRate => b"DISCOUNT",
        YieldToMaturity => b"YTM",
        AbsolutePrepaymentSpeed => b"ABS",
        ConstantPrepaymentPenalty => b"CPP",
        ConstantPrepaymentRate => b"CPR",
        ConstantPrepaymentYield => b"CPY",
        FinalCPROfHomeEquityPrepaymentCurve => b"HEP",
        PercentOfManufacturedHousingPrepaymentCurve => b"MHP",
        MonthlyPrepaymentRate => b"MPR",
        PercentOfProspectusPrepaymentCurve => b"PPC",
        PercentOfBMAPrepaymentCurve => b"PSA",
        SingleMonthlyMortality => b"SMM",
    } Other,
    FIELD_TYPE [REQUIRED_AND_NOT_REQUIRED,BYTES] RequiredStipulationTypeFieldType NotRequiredStipulationTypeFieldType
);

define_enum_field_type!(
    FIELD StrikePriceBoundaryMethod {
        LessThanUnderlyingPriceIsInTheMoney => b"1",
        LessThanOrEqualToTheUnderlyingPriceIsInTheMoney => b"2",
        EqualToTheUnderlyingPriceIsInTheMoney => b"3",
        GreaterThanOrEqualToUnderlyingPriceIsInTheMoney => b"4",
        GreaterThanUnderlyingIsInTheMoney => b"5",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] StrikePriceBoundaryMethodFieldType
);

define_enum_field_type!(
    FIELD StrikePriceDeterminationMethod {
        FixedStrike => 1,
        StrikeSetAtExpirationToUnderlyingOrOtherValue => 2,
        StrikeSetToAverageOfUnderlyingSettlementPriceAcrossTheLifeOfTheOption => 3,
        StrikeSetToOptimalValue => 4,
    } Reserved100Plus => WITH_MINIMUM 100,
    FIELD_TYPE [NOT_REQUIRED] StrikePriceDeterminationMethodFieldType
);

define_enum_field_type!(
    FIELD SymbolSfx {
        EUCPWithLumpSumInterestRatherThanDiscountPrice => b"CD",
        WhenIssued => b"WI",
    } Other,
    FIELD_TYPE [REQUIRED_AND_NOT_REQUIRED,BYTES] RequiredSymbolSfxFieldType NotRequiredSymbolSfxFieldType
);

define_enum_field_type!(
    FIELD TimeInForce {
        Day => b"0",
        GoodTillCancel => b"1",
        AtTheOpening => b"2",
        ImmediateOrCancel => b"3",
        FillOrKill => b"4",
        GoodTillCrossing => b"5",
        GoodTillDate => b"6",
        AtTheClose => b"7",
        GoodThroughCrossing => b"8",
        AtCrossing => b"9",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_CHAR] TimeInForceFieldType
);

define_enum_field_type!(
    FIELD TimeUnit {
        Hour => b"H",
        Minute => b"Min",
        Second => b"S",
        Day => b"D",
        Week => b"Wk",
        Month => b"Mo",
        Year => b"Yr",
    } Other,
    FIELD_TYPE [REQUIRED_AND_NOT_REQUIRED,BYTES] RequiredTimeUnitFieldType NotRequiredTimeUnitFieldType
);

define_enum_field_type!(
    FIELD UnderlyingCashType {
        Fixed => b"FIXED",
        Diff => b"DIFF",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_STRING] UnderlyingCashTypeFieldType
);

define_enum_field_type!(
    FIELD UnderlyingFXRateCalc {
        Divide => b"D",
        Multiply => b"M",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_CHAR] UnderlyingFXRateCalcFieldType
);

define_enum_field_type!(
    FIELD UnderlyingPriceDeterminationMethod {
        Regular => b"1",
        SpecialReference => b"2",
        OptimalValue => b"3",
        AverageValue => b"4",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] UnderlyingPriceDeterminationMethodFieldType
);

define_enum_field_type!(
    FIELD UnderlyingSettlementType {
        TPlus1 => b"2",
        TPlus3 => b"4",
        TPlus4 => b"5",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_INT] UnderlyingSettlementTypeFieldType
);

define_enum_field_type!(
    FIELD UnitOfMeasure {
        BillionCubicFeet => b"Bcf",
        MillionBarrels => b"MMbbl", //Deprecated in FIX 5.0 SP1.
        OneMillionBTU => b"MMBtu",
        MegawattHours => b"MWh",
        Barrels => b"Bbl",
        Bushels => b"Bu",
        Pounds => b"lbs",
        Gallons => b"Gal",
        TroyOunces => b"oz_tr",
        MetricTons => b"t", //Tonne
        Tons => b"tn", //US
        USDollars => b"USD",
        Allowances => b"Alw",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_STRING] UnitOfMeasureFieldType
);

define_enum_field_type!(
    FIELD ValuationMethod {
        CDSStyleCollateralizationOfMarketToMarketAndCoupon => b"CDS",
        CDSInDelivery => b"CDSD",
        PremiumStyle => b"EQTY",
        FuturesStyleMarkToMarket => b"FUT",
        FuturesStyleWithAnAttachedCashAdjustment => b"FUTDA",
    },
    FIELD_TYPE [NOT_REQUIRED,MUST_BE_STRING] ValuationMethodFieldType
);
