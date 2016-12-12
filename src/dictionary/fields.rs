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

use field_type::{StringFieldType,DataFieldType,NoneFieldType,RepeatingGroupFieldType,SeqNumFieldType,UTCTimestampFieldType,IntFieldType,SideFieldType,BoolTrueOrBlankFieldType};
use message::{REQUIRED,NOT_REQUIRED};
use rule::Rule;

define_fields!(
    Account: StringFieldType = b"1",
    BeginSeqNo: SeqNumFieldType = b"7",
    ClOrdID: StringFieldType = b"11",
    Currency: StringFieldType = b"15", //Currency
    EndSeqNo: SeqNumFieldType = b"16",
    HandInst: StringFieldType = b"21", //Char, TODO: limited choices.
    SecurityIDSource: StringFieldType = b"22", //TODO: Limited choices.
    MsgSeqNum: SeqNumFieldType = b"34", //TODO: Special field probably might be better off built into the parser.
    NewSeqNo: SeqNumFieldType = b"36",
    OrderQty: StringFieldType = b"38", //Qty
    OrdType: StringFieldType = b"40", //Char, TODO: limited choices.
    PossDupFlag: BoolTrueOrBlankFieldType = b"43",
    Price: StringFieldType = b"44", //Price
    RefSeqNum: SeqNumFieldType = b"45",
    SecurityID: StringFieldType = b"48",
    SenderCompID: StringFieldType = b"49",
    SenderSubID: StringFieldType = b"50",
    SendingTime: UTCTimestampFieldType = b"52",
    SideField: SideFieldType = b"54",
    Symbol: StringFieldType = b"55",
    TargetCompID: StringFieldType = b"56",
    TargetSubID: StringFieldType = b"57",
    Text: StringFieldType = b"58",
    TimeInForce: StringFieldType = b"59", //Char, TODO: limited choices.
    TransactTime: StringFieldType = b"60", //UTCTimestamp
    SettlType: StringFieldType = b"63", //TODO: Limited choices.
    SettlDate: StringFieldType = b"64", //LocalMktDate
    NoOrders: RepeatingGroupFieldType<Order> = b"73",
    NoAllocs: RepeatingGroupFieldType<Alloc> = b"78",
    AllocAccount: StringFieldType = b"79",
    Signature: DataFieldType = b"89" => Rule::ConfirmPreviousTag{ previous_tag: SignatureLength::tag() },
    SecureDataLen: NoneFieldType = b"90" => Rule::PrepareForBytes{ bytes_tag: SecureData::tag() },
    SecureData: DataFieldType = b"91" => Rule::ConfirmPreviousTag{ previous_tag: SecureDataLen::tag() },
    SignatureLength: NoneFieldType = b"93" => Rule::PrepareForBytes{ bytes_tag: Signature::tag() },
    RawDataLength: NoneFieldType = b"95" => Rule::PrepareForBytes{ bytes_tag: RawData::tag() },
    RawData: DataFieldType = b"96" => Rule::ConfirmPreviousTag{ previous_tag: RawDataLength::tag() },
    PossResend: StringFieldType = b"97", //Bool
    EncryptMethod: StringFieldType = b"98",
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
    CashOrderQty: StringFieldType = b"152", //Qty
    XmlDataLen: NoneFieldType = b"212" => Rule::PrepareForBytes{ bytes_tag: XmlData::tag() },
    XmlData: DataFieldType = b"213" => Rule::ConfirmPreviousTag{ previous_tag: XmlDataLen::tag() },
    MessageEncoding: StringFieldType = b"347",
    EncodedTextLen: NoneFieldType = b"354" => Rule::PrepareForBytes{ bytes_tag: EncodedText::tag() },
    EncodedText: DataFieldType = b"355" => Rule::ConfirmPreviousTag{ previous_tag: EncodedTextLen::tag() },
    LastMsgSeqNumProcessed: SeqNumFieldType = b"369",
    RefTagID: StringFieldType = b"371", //int
    RefMsgType: StringFieldType = b"372",
    SessionRejectReason: StringFieldType = b"373", //int
    BusinessRejectRefID: StringFieldType = b"379",
    BusinessRejectReason: StringFieldType = b"380", //int //TODO: limited choices.
    MaxMessageSize: StringFieldType = b"383", //Length
    NoMsgTypeGrp: RepeatingGroupFieldType<MsgTypeGrp> = b"384",
    MsgDirection: StringFieldType = b"385", //Char
    TestMessageIndicator: StringFieldType = b"464", //Bool
    Username: StringFieldType = b"553",
    Password: StringFieldType = b"554",
    NoHops: RepeatingGroupFieldType<HopGrp> = b"627",
    HopCompID: StringFieldType = b"628",
    HopSendingTime: StringFieldType = b"629", //UTCTimestamp
    HopRefID: SeqNumFieldType = b"630",
    NextExpectedMsgSeqNum: SeqNumFieldType = b"789",
    NewPassword: StringFieldType = b"925",
    ApplVerID: StringFieldType = b"1128", //TODO: limited choices.
    CstmApplVerID: StringFieldType = b"1129",
    RefApplVerID: StringFieldType = b"1130",
    RefCstmApplVerID: StringFieldType = b"1131",
    DefaultApplVerID: StringFieldType = b"1137", //TODO: limited choices.
    ApplExtID: StringFieldType = b"1156", //int
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
    NoRateSources: RepeatingGroupFieldType<RateSource> = b"1445",
    RateSourceField: StringFieldType = b"1446", //int
    RateSourceType: StringFieldType = b"1447", //int
    ReferencePage: StringFieldType = b"1448",
);

//Repeating Groups

define_message!(HopGrp {
    REQUIRED, hop_comp_id: HopCompID,
    NOT_REQUIRED, hop_sending_time: HopSendingTime,
    NOT_REQUIRED, hop_ref_id: HopRefID,
});

define_message!(Alloc {
    REQUIRED, alloc_account: AllocAccount,
});

define_message!(Order {
    REQUIRED, cl_ord_id: ClOrdID,
    NOT_REQUIRED, allocs: NoAllocs,
});

define_message!(RateSource {
    REQUIRED, rate_source: RateSourceField,
    REQUIRED, rate_source_type: RateSourceType,
    NOT_REQUIRED, reference_page: ReferencePage, //Required if RateSource = other.
});

define_message!(MsgTypeGrp {
    REQUIRED, ref_msg_type: RefMsgType,
    REQUIRED, msg_direction: MsgDirection,
    NOT_REQUIRED, ref_appl_ver_id: RefApplVerID,
    NOT_REQUIRED, ref_appl_ext_id: RefApplExtID,
    NOT_REQUIRED, ref_cstm_appl_ver_id: RefCstmApplVerID,
    NOT_REQUIRED, default_ver_indicator: DefaultVerIndicator,
});
