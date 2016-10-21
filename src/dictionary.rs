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

use std::any::Any;
use std::collections::{HashMap,HashSet};
use std::io::Write;

use constant::{TAG_END,VALUE_END};
use field::Field;
use field_type::{FieldType,StringFieldType,DataFieldType,NoneFieldType,RepeatingGroupFieldType};
use message::{REQUIRED,NOT_REQUIRED,Meta,Message};
use rule::Rule;

#[macro_export]
macro_rules! define_dictionary {
    ( $( $msg_type:expr => $msg:ty : $msg_enum:ident ),* $(),* ) => {
        fn build_dictionary() -> HashMap<&'static [u8],Box<Message>> {
            let mut message_dictionary: HashMap<&'static [u8],Box<Message>> = HashMap::new();
            $( message_dictionary.insert($msg_type,Box::new(<$msg as Default>::default())); )*

            message_dictionary
        }

        #[allow(dead_code)]
        enum MessageEnum
        {
            $( $msg_enum($msg), )*
        };

        #[allow(dead_code)]
        fn message_to_enum(message: &Message) -> MessageEnum {
            if false {
            }
            $( else if message.as_any().is::<$msg>() {
                //TODO: Avoid the clone.
                return MessageEnum::$msg_enum(message.as_any().downcast_ref::<$msg>().unwrap().clone());
            } )*

            panic!("Unsupported message");
        }
    };
}

//TODO: Maybe put the tag number first here. It'll be more consistent and be easier to read.
define_fields!(
    Account: StringFieldType = b"1",
    BeginSeqNo: StringFieldType = b"7", //SeqNum
    ClOrdID: StringFieldType = b"11",
    Currency: StringFieldType = b"15", //Currency
    EndSeqNo: StringFieldType = b"16", //SeqNum
    HandInst: StringFieldType = b"21", //Char, TODO: limited choices.
    SecurityIDSource: StringFieldType = b"22", //TODO: Limited choices.
    MsgSeqNum: StringFieldType = b"34", //TODO: Special field probably should be built into the parser.
    NewSeqNo: StringFieldType = b"36", //SeqNum
    OrderQty: StringFieldType = b"38", //Qty
    OrdType: StringFieldType = b"40", //Char, TODO: limited choices.
    PossDupFlag: StringFieldType = b"43", //Bool
    Price: StringFieldType = b"44", //Price
    RefSeqNum: StringFieldType = b"45", //SeqNum
    SecurityID: StringFieldType = b"48",
    SenderCompID: StringFieldType = b"49",
    SenderSubID: StringFieldType = b"50",
    SendingTime: StringFieldType = b"52", //UTCTimestamp
    Side: StringFieldType = b"54", //Char, TODO: limited choices.
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
    HeartBtInt: StringFieldType = b"108",
    MinQty: StringFieldType = b"110", //Qty
    MaxFloor: StringFieldType = b"111", //Qty
    TestReqID: StringFieldType = b"112",
    OnBehalfOfCompID: StringFieldType = b"115",
    OnBehalfOfSubID: StringFieldType = b"116",
    OrigSendingTime: StringFieldType = b"122", //UTCTimestamp
    GapFillFlag: StringFieldType = b"123", //bool
    DeliverToCompID: StringFieldType = b"128",
    DeliverToSubID: StringFieldType = b"129",
    BidSize: StringFieldType = b"134", //Qty
    ResetSeqNumFlag: StringFieldType = b"141", //Bool
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
    LastMsgSeqNumProcessed: StringFieldType = b"369", //SeqNum
    RefTagID: StringFieldType = b"371", //int
    RefMsgType: StringFieldType = b"372",
    SessionRejectReason: StringFieldType = b"373", //int
    MaxMessageSize: StringFieldType = b"383", //Length
    NoMsgTypeGrp: RepeatingGroupFieldType<MsgTypeGrp> = b"384",
    MsgDirection: StringFieldType = b"385", //Char
    TestMessageIndicator: StringFieldType = b"464", //Bool
    Username: StringFieldType = b"553",
    Password: StringFieldType = b"554",
    NoHops: RepeatingGroupFieldType<HopGrp> = b"627",
    HopCompID: StringFieldType = b"628",
    HopSendingTime: StringFieldType = b"629", //UTCTimestamp
    HopRefID: StringFieldType = b"630", //SeqNum
    NextExpectedMsgSeqNum: StringFieldType = b"789", //SeqNum
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

//TODO: Support message components that should be embedded as is....

#[macro_export]
macro_rules! define_fixt_message {
    ( $message_name:ident { $( $field_required:expr, $field_name:ident : $field_type:ty),* $(),* } ) => {
        define_message!($message_name {
            //Standard Header
            //Note: BeginStr, BodyLength, and MsgType are built into parser.
            NOT_REQUIRED, appl_ver_id: ApplVerID,
            NOT_REQUIRED, appl_ext_id: ApplExtID,
            NOT_REQUIRED, cstm_appl_ver_id: CstmApplVerID,
            REQUIRED, sender_comp_id: SenderCompID,
            REQUIRED, target_comp_id: TargetCompID,
            NOT_REQUIRED, on_behalf_of_comp_id: OnBehalfOfCompID,
            NOT_REQUIRED, deliver_to_comp_id: DeliverToCompID,
            NOT_REQUIRED, secure_data_len: SecureDataLen,
            NOT_REQUIRED, secure_data: SecureData,
            REQUIRED, msg_seq_num: MsgSeqNum,
            NOT_REQUIRED, sender_sub_id: SenderSubID,
            NOT_REQUIRED, sender_location_id: SenderLocationID,
            NOT_REQUIRED, target_sub_id: TargetSubID,
            NOT_REQUIRED, target_location_id: TargetLocationID,
            NOT_REQUIRED, on_behalf_of_sub_id: OnBehalfOfSubID,
            NOT_REQUIRED, on_behalf_of_location_id: OnBehalfOfLocationID,
            NOT_REQUIRED, deliver_to_sub_id: DeliverToSubID,
            NOT_REQUIRED, deliver_to_location_id: DeliverToLocationID,
            NOT_REQUIRED, poss_dup_flag: PossDupFlag,
            NOT_REQUIRED, poss_resend: PossResend,
            REQUIRED, sending_time: SendingTime,
            NOT_REQUIRED, orig_sending_time: OrigSendingTime,
            NOT_REQUIRED, xml_data_len: XmlDataLen,
            NOT_REQUIRED, xml_data: XmlData,
            NOT_REQUIRED, message_encoding: MessageEncoding,
            NOT_REQUIRED, last_msg_seq_num_processed: LastMsgSeqNumProcessed,
            NOT_REQUIRED, hops: NoHops,

            //Other
            $( $field_required, $field_name : $field_type, )*

            //Standard Footer
            //Note: Checksum is built into parser.
            NOT_REQUIRED, signature_length: SignatureLength,
            NOT_REQUIRED, signature: Signature,
        });
    };
}

//TODO: Support embedding redundant blocks into messages. ie. OrderQtyData.
//TODO: all of the following messages are incomplete in one way or another right now.

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

//FIXT Administrative Messages

define_fixt_message!(Heartbeat {
    NOT_REQUIRED, test_req_id: TestReqID,
});

define_fixt_message!(Logon {
    REQUIRED, encrypt_method: EncryptMethod,
    REQUIRED, heart_bt_int: HeartBtInt,
    NOT_REQUIRED, raw_data_length: RawDataLength,
    NOT_REQUIRED, raw_data: RawData,
    NOT_REQUIRED, reset_seq_num_flag: ResetSeqNumFlag,
    NOT_REQUIRED, next_expected_msg_seq_num: NextExpectedMsgSeqNum,
    NOT_REQUIRED, max_message_size: MaxMessageSize,
    NOT_REQUIRED, msg_type_grp: NoMsgTypeGrp,
    NOT_REQUIRED, test_message_indicator: TestMessageIndicator,
    NOT_REQUIRED, username: Username,
    NOT_REQUIRED, password: Password,
    NOT_REQUIRED, new_password: NewPassword,
    NOT_REQUIRED, encrypted_password_method: EncryptedPasswordMethod,
    NOT_REQUIRED, encrypted_password_len: EncryptedPasswordLen,
    NOT_REQUIRED, encrypted_password: EncryptedPassword,
    NOT_REQUIRED, encrypted_new_password_len: EncryptedNewPasswordLen,
    NOT_REQUIRED, encrypted_new_password: EncryptedNewPassword,
    NOT_REQUIRED, session_status: SessionStatus,
    REQUIRED, default_appl_ver_id: DefaultApplVerID,
    NOT_REQUIRED, default_appl_ext_id: DefaultApplExtID,
    NOT_REQUIRED, default_cstm_appl_ver_id: DefaultCstmApplVerID,
    NOT_REQUIRED, text: Text,
    NOT_REQUIRED, encoded_text_len: EncodedTextLen,
    NOT_REQUIRED, encoded_text: EncodedText,
});

define_fixt_message!(TestRequest {
    REQUIRED, test_req_id: TestReqID,
});

define_fixt_message!(ResendRequest {
    REQUIRED, begin_seq_no: BeginSeqNo,
    REQUIRED, end_seq_no: EndSeqNo,
});

define_fixt_message!(Reject {
    REQUIRED, ref_seq_num: RefSeqNum,
    NOT_REQUIRED, ref_tag_id: RefTagID,
    NOT_REQUIRED, ref_msg_type: RefMsgType,
    NOT_REQUIRED, ref_appl_ver_id: RefApplVerID,
    NOT_REQUIRED, ref_appl_ext_id: RefApplExtID,
    NOT_REQUIRED, ref_cstm_appl_ver_id: RefCstmApplVerID,
    NOT_REQUIRED, session_reject_reason: SessionRejectReason,
    NOT_REQUIRED, text: Text,
    NOT_REQUIRED, encoded_text_len: EncodedTextLen,
    NOT_REQUIRED, encoded_text: EncodedText,
});

define_fixt_message!(SequenceReset {
    NOT_REQUIRED, gap_fill_flag: GapFillFlag,
    REQUIRED, new_seq_no: NewSeqNo,
});

define_fixt_message!(Logout {
    NOT_REQUIRED, session_status: SessionStatus,
    NOT_REQUIRED, text: Text,
    NOT_REQUIRED, encoded_text_len: EncodedTextLen,
    NOT_REQUIRED, encoded_text: EncodedText,
});

//Other Messages

define_fixt_message!(NewOrderSingle {
    REQUIRED, cl_ord_id: ClOrdID,
    /*NOT_REQUIRED, secondary_cl_ord_id: SecondaryClOrdID,
    NOT_REQUIRED, cl_ord_link_id: ClOrdLinkID,
    NOT_REQUIRED, parties: NoParties,
    NOT_REQUIRED, trade_origination_date: TradeOriginationDate,
    NOT_REQUIRED, trade_date: TradeDate,*/
    NOT_REQUIRED, account: Account,
    /*NOT_REQUIRED, acct_id_source: AcctIDSource,
    NOT_REQUIRED, account_type: AccountType,
    NOT_REQUIRED, day_booking_inst: DayBookingInst,
    NOT_REQUIRED, booking_unit: BookingUnit,
    NOT_REQUIRED, prealloc_method: PreallocMethod,
    NOT_REQUIRED, alloc_id: AllocID,
    NOT_REQUIRED, pre_alloc_grp: NoPreAllocGrp,*/
    NOT_REQUIRED, settl_type: SettlType,
    NOT_REQUIRED, settl_date: SettlDate,
    /*NOT_REQUIRED, cash_margin: CashMargin,
    NOT_REQUIRED, clearing_free_indicator: ClearingFreeIndicator,*/
    NOT_REQUIRED, hand_inst: HandInst,
    /*NOT_REQUIRED, exec_inst: ExecInst,*/
    NOT_REQUIRED, min_qty: MinQty,
    /*NOT_REQUIRED, match_increment: MatchIncrement,
    NOT_REQUIRED, max_price_levels: MaxPriceLevels,
    NOT_REQUIRED, display_instruction: NoDisplayInstruction,*/
    NOT_REQUIRED, max_floor: MaxFloor,
    /*NOT_REQUIRED, ex_destination: ExDestination,
    NOT_REQUIRED, ex_destination_id_source: ExDestinationIDSource,
    NOT_REQUIRED, trdg_ses_grp: NoTrdgSesGrp,
    NOT_REQUIRED, process_code: ProcessCode,
    REQUIRED, instrument: NoInstrument,*/
        REQUIRED, symbol: Symbol, //TODO: Part of the Instrument block.
        REQUIRED, security_id: SecurityID, //TODO: Part of the Instrument block.
        REQUIRED, security_id_source: SecurityIDSource, //TODO: Part of the Instrument block.
    /*NOT_REQUIRED, financing_details: NoFinancingDetails,
    NOT_REQUIRED, und_instrmt_grp: NoUndInstrmtGrp,
    NOT_REQUIRED, prev_close_px: PrevClosePx,*/
    REQUIRED, side: Side,
    /*NOT_REQUIRED, locate_reqd: LocateReqd,*/
    REQUIRED, transact_time: TransactTime,
    /*NOT_REQUIRED, stipulations: NoStipulations,
    NOT_REQUIRED, qty_type: QtyType,*/
    REQUIRED, order_qty: OrderQty, //TODO: One and only one of OrderQty, CashOrderQty or OrderPrecent should be specified.
    /*NOT_REQUIRED, cash_order_qty: CashOrderQty,
    NOT_REQUIRED, order_precent: OrderPercent,
    NOT_REQUIRED, rounding_direction: RoundingDirection,
    NOT_REQUIRED, rounding_modulus: RoundingModulus,*/
    REQUIRED, ord_type: OrdType,
    /*NOT_REQUIRED, price_type: PriceType,*/
    NOT_REQUIRED, price: Price,
    /*NOT_REQUIRED, price_protection_scope: PriceProtectionScope,
    NOT_REQUIRED, stop_px: StopPx,
    NOT_REQUIRED, triggering_instruction: NoTriggeringInstruction,
    NOT_REQUIRED, spread_or_benchmark_curve_data: NoSpreadOrBenchmarkCurveData,
    NOT_REQUIRED, yield_data: NoYieldData,*/
    NOT_REQUIRED, currency: Currency,
    /*NOT_REQUIRED, compliance_id: ComplianceID,
    NOT_REQUIRED, solicited_flag: SolicitedFlag,
    NOT_REQUIRED, ioi_id: IOIID,
    NOT_REQUIRED, quote_id: QuoteID,*/
    NOT_REQUIRED, time_in_force: TimeInForce,
    /*NOT_REQUIRED, effective_time: EffectiveTime,
    NOT_REQUIRED, expire_data: ExpireDate,
    NOT_REQUIRED, expire_time: ExpireTime,
    NOT_REQUIRED, gt_booking_inst: GTBoookingInst,
    NOT_REQUIRED, commission_data: NoCommissionData,
    NOT_REQUIRED, order_capacity: OrderCapacity,
    NOT_REQUIRED, order_restrictions: OrderRestrictions,
    NOT_REQUIRED, pre_trade_anonymity: PreTradeAnonymity,
    NOT_REQUIRED, cust_order_capacity: CustOrderCapacity,
    NOT_REQUIRED, forex_req: ForexReq,
    NOT_REQUIRED, settl_currency: SettlCurrency,
    NOT_REQUIRED, booking_type: BookingType,
    NOT_REQUIRED, text: Text,
    NOT_REQUIRED, encoded_text_len: EncodedTextLen,
    NOT_REQUIRED, encoded_text: EncodedText,
    NOT_REQUIRED, settl_date2: SettlDate2,
    NOT_REQUIRED, order_qty2: OrderQty2,
    NOT_REQUIRED, price2: Price2,
    NOT_REQUIRED, position_effect: PositionEffect,
    NOT_REQUIRED, covered_or_uncovered: CoveredOrUncovered,
    NOT_REQUIRED, max_show: MaxShow,
    NOT_REQUIRED, peg_instructions: NoPegInstructions,
    NOT_REQUIRED, discretion_instructions: NoDiscretionInstructions,
    NOT_REQUIRED, target_strategy: TargetStrategy,
    NOT_REQUIRED, strategy_parameters_grp: NoStrategyParametersGrp,
    NOT_REQUIRED, target_strategy_parameters: TargetStrategyParameters,
    NOT_REQUIRED, participation_rate: ParticipationRate,
    NOT_REQUIRED, cancellation_rights: CancellationRights,
    NOT_REQUIRED, money_laundering_status: MoneyLaunderingStatus,
    NOT_REQUIRED, regist_id: RegistID,
    NOT_REQUIRED, designation: Designation,
    NOT_REQUIRED, manual_order_indicator: ManualOrderIndicator,
    NOT_REQUIRED, cust_directed_order: CustDirectedOrder,
    NOT_REQUIRED, received_dept_id: ReceivedDeptID,
    NOT_REQUIRED, cust_order_handling_inst: CustOrderHandlingInst,
    NOT_REQUIRED, order_handling_inst_source: OrderHandlingInstSource,
    NOT_REQUIRED, trd_reg_timestamps: NoTrdRegTimestamps,
    NOT_REQUIRED, ref_order_id: RefOrderID,
    NOT_REQUIRED, ref_order_id_source: RefOrderIDSource,*/
});
