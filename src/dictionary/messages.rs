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

use dictionary::fields::*;
use field::Field;
use field_type::FieldType;
use fixt::message::FIXTMessage;
use message::{REQUIRED,NOT_REQUIRED,Message,Meta,SetValueError};
use rule::Rule;

pub struct NullMessage {
}

impl Message for NullMessage {
    fn first_field(&self) -> &'static [u8] {
        unimplemented!();
    }

    fn field_count(&self) -> usize {
        unimplemented!();
    }

    fn fields(&self) -> HashMap<&'static [u8],Rule> {
        unimplemented!();
    }

    fn required_fields(&self) -> HashSet<&'static [u8]> {
        unimplemented!();
    }

    fn conditional_required_fields(&self) -> Vec<&'static [u8]> {
        unimplemented!();
    }

    fn set_meta(&mut self,_meta: Meta) {
        unimplemented!();
    }

    fn set_value(&mut self,_key: &[u8],_value: &[u8]) -> Result<(),SetValueError> {
        unimplemented!();
    }

    fn set_groups(&mut self,_key: &[u8],_group: &[Box<Message>]) -> bool {
        unimplemented!();
    }

    fn as_any(&self) -> &Any {
        unimplemented!();
    }

    fn as_any_mut(&mut self) -> &mut Any {
        unimplemented!();
    }

    fn new_into_box(&self) -> Box<Message + Send> {
        unimplemented!();
    }

    fn msg_type_header(&self) -> Vec<u8> {
        Vec::new()
    }

    fn read_body(&self,_buf: &mut Vec<u8>) -> usize {
        unimplemented!();
    }
}

impl FIXTMessage for NullMessage {
    fn new_into_box(&self) -> Box<FIXTMessage + Send> {
        unimplemented!();
    }

    fn msg_type(&self) -> &'static [u8] {
        unimplemented!();
    }

    fn msg_seq_num(&self) -> <<MsgSeqNum as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn sender_comp_id(&self) -> &<<SenderCompID as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn target_comp_id(&self) -> &<<TargetCompID as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn is_poss_dup(&self) -> bool {
        unimplemented!();
    }

    fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn setup_fixt_session_header(&mut self,
                                 _msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
                                 _sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
                                 _target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type) {
        unimplemented!();
    }
}

//FIXT Administrative Messages
define_fixt_message!(Heartbeat: b"0" => {
    NOT_REQUIRED, test_req_id: TestReqID,
});

define_fixt_message!(Logon: b"A" => {
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

define_fixt_message!(TestRequest: b"1" => {
    REQUIRED, test_req_id: TestReqID,
});

define_fixt_message!(ResendRequest: b"2" => {
    REQUIRED, begin_seq_no: BeginSeqNo,
    REQUIRED, end_seq_no: EndSeqNo,
});

define_fixt_message!(Reject: b"3" => {
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

define_fixt_message!(SequenceReset: b"4" => {
    NOT_REQUIRED, gap_fill_flag: GapFillFlag,
    REQUIRED, new_seq_no: NewSeqNo,
});

define_fixt_message!(Logout: b"5" => {
    NOT_REQUIRED, session_status: SessionStatus,
    NOT_REQUIRED, text: Text,
    NOT_REQUIRED, encoded_text_len: EncodedTextLen,
    NOT_REQUIRED, encoded_text: EncodedText,
});

//Other Messages

define_fixt_message!(Email: b"C" => {
    REQUIRED, email_thread_id: EmailThreadID,
    REQUIRED, email_type: EmailType,
    NOT_REQUIRED, orig_time: OrigTime,
    REQUIRED, subject: Subject,
    NOT_REQUIRED, encoded_subject_len: EncodedSubjectLen,
    NOT_REQUIRED, encoded_subject: EncodedSubject,
    NOT_REQUIRED, no_routing_ids: NoRoutingIDs,
    NOT_REQUIRED, no_related_sym: NoRelatedSym,
    NOT_REQUIRED, no_underlyings: NoUnderlyings,
    NOT_REQUIRED, no_legs: NoLegs,
    NOT_REQUIRED, order_id: OrderID,
    NOT_REQUIRED, cl_ord_id: ClOrdID,
    REQUIRED, no_lines_of_text: NoLinesOfText,
    NOT_REQUIRED, raw_data_length: RawDataLength,
    NOT_REQUIRED, raw_data: RawData,
});

define_fixt_message!(BusinessMessageReject: b"j" => {
    NOT_REQUIRED, ref_seq_num: RefSeqNum,
    REQUIRED, ref_msg_type: RefMsgType,
    NOT_REQUIRED, ref_appl_ver_id: RefApplVerID,
    NOT_REQUIRED, ref_appl_ext_id: RefApplExtID,
    NOT_REQUIRED, ref_cstm_appl_ver_id: RefCstmApplVerID,
    NOT_REQUIRED, business_reject_ref_id: BusinessRejectRefID,
    REQUIRED, business_reject_reason: BusinessRejectReason,
    NOT_REQUIRED, text: Text,
    NOT_REQUIRED, encoded_text_len: EncodedTextLen,
    NOT_REQUIRED, encoded_text: EncodedText,
});

define_fixt_message!(NewOrderSingle: b"D" => {
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
    NOT_REQUIRED, handl_inst: HandlInst,
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
    REQUIRED, side: SideField,
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
