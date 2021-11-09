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

use crate::dictionary::fields::*;
use crate::field::Field;
use crate::field_tag::{self, FieldTag};
use crate::field_type::FieldType;
use crate::fix_version::FIXVersion;
use crate::fixt;
use crate::fixt::message::FIXTMessage;
use crate::message::{self, Message, Meta, SetValueError, NOT_REQUIRED, REQUIRED};
use crate::message_version::{self, MessageVersion};

pub struct NullMessage {}

impl Message for NullMessage {
    fn conditional_required_fields(&self, _version: MessageVersion) -> Vec<FieldTag> {
        unimplemented!();
    }

    fn meta(&self) -> &Option<Meta> {
        unimplemented!();
    }

    fn set_meta(&mut self, _meta: Meta) {
        unimplemented!();
    }

    fn set_value(&mut self, _key: FieldTag, _value: &[u8]) -> Result<(), SetValueError> {
        unimplemented!();
    }

    fn set_groups(&mut self, _key: FieldTag, _group: Vec<Box<dyn Message>>) -> bool {
        unimplemented!();
    }

    fn as_any(&self) -> &dyn Any {
        unimplemented!();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        unimplemented!();
    }

    fn new_into_box(&self) -> Box<dyn Message + Send> {
        unimplemented!();
    }

    fn msg_type_header(&self) -> &'static [u8] {
        b""
    }

    fn read_body(
        &self,
        _fix_version: FIXVersion,
        _message_version: MessageVersion,
        _buf: &mut Vec<u8>,
    ) -> usize {
        unimplemented!();
    }
}

impl FIXTMessage for NullMessage {
    fn new_into_box(&self) -> Box<dyn FIXTMessage + Send> {
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

    fn set_is_poss_dup(&mut self, _is_poss_dup: bool) {
        unimplemented!();
    }

    fn sending_time(&self) -> <<SendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn orig_sending_time(&self) -> <<OrigSendingTime as Field>::Type as FieldType>::Type {
        unimplemented!();
    }

    fn set_orig_sending_time(
        &mut self,
        _orig_sending_time: <<OrigSendingTime as Field>::Type as FieldType>::Type,
    ) {
        unimplemented!();
    }

    fn setup_fixt_session_header(
        &mut self,
        _msg_seq_num: Option<<<MsgSeqNum as Field>::Type as FieldType>::Type>,
        _sender_comp_id: <<SenderCompID as Field>::Type as FieldType>::Type,
        _target_comp_id: <<TargetCompID as Field>::Type as FieldType>::Type,
    ) {
        unimplemented!();
    }
}

//FIXT Administrative Messages
define_fixt_message!(Heartbeat: ADMIN b"0" => {
    NOT_REQUIRED, test_req_id: TestReqID [FIX40..],
});

define_fixt_message!(Logon: ADMIN b"A" => {
    REQUIRED, encrypt_method: EncryptMethod [FIX40..],
    REQUIRED, heart_bt_int: HeartBtInt [FIX40..],
    NOT_REQUIRED, raw_data_length: RawDataLength [FIX40..],
    NOT_REQUIRED, raw_data: RawData [FIX40..],
    NOT_REQUIRED, reset_seq_num_flag: ResetSeqNumFlag [FIX41..],
    NOT_REQUIRED, next_expected_msg_seq_num: NextExpectedMsgSeqNum [FIX44..],
    NOT_REQUIRED, max_message_size: MaxMessageSize [FIX42..],
    NOT_REQUIRED, no_msg_types: NoMsgTypeGrp [FIX42..],
    NOT_REQUIRED, test_message_indicator: TestMessageIndicator [FIX43..],
    NOT_REQUIRED, username: Username [FIX43..],
    NOT_REQUIRED, password: Password [FIX43..],
    NOT_REQUIRED, new_password: NewPassword [FIX50SP1..],
    NOT_REQUIRED, encrypted_password_method: EncryptedPasswordMethod [FIX50SP1..],
    NOT_REQUIRED, encrypted_password_len: EncryptedPasswordLen [FIX50SP1..],
    NOT_REQUIRED, encrypted_password: EncryptedPassword [FIX50SP1..],
    NOT_REQUIRED, encrypted_new_password_len: EncryptedNewPasswordLen [FIX50SP1..],
    NOT_REQUIRED, encrypted_new_password: EncryptedNewPassword [FIX50SP1..],
    NOT_REQUIRED, session_status: SessionStatus [FIX50SP1..],
    REQUIRED, default_appl_ver_id: DefaultApplVerID [FIX50..],
    NOT_REQUIRED, default_appl_ext_id: DefaultApplExtID [FIX50SP1..],
    NOT_REQUIRED, default_cstm_appl_ver_id: DefaultCstmApplVerID [FIX50SP1..],
    NOT_REQUIRED, text: Text [FIX50SP1..],
    NOT_REQUIRED, encoded_text_len: EncodedTextLen [FIX50SP1..],
    NOT_REQUIRED, encoded_text: EncodedText [FIX50SP1..],
});

define_fixt_message!(TestRequest: ADMIN b"1" => {
    REQUIRED, test_req_id: TestReqID [FIX40..],
});

define_fixt_message!(ResendRequest: ADMIN b"2" => {
    REQUIRED, begin_seq_no: BeginSeqNo [FIX40..],
    REQUIRED, end_seq_no: EndSeqNo [FIX40..],
});

define_fixt_message!(Reject: ADMIN b"3" => {
    REQUIRED, ref_seq_num: RefSeqNum [FIX40..],
    NOT_REQUIRED, ref_tag_id: RefTagID [FIX42..],
    NOT_REQUIRED, ref_msg_type: RefMsgType [FIX42..],
    NOT_REQUIRED, ref_appl_ver_id: RefApplVerID [FIX50SP1..],
    NOT_REQUIRED, ref_appl_ext_id: RefApplExtID [FIX50SP1..],
    NOT_REQUIRED, ref_cstm_appl_ver_id: RefCstmApplVerID [FIX50SP1..],
    NOT_REQUIRED, session_reject_reason: SessionRejectReason [FIX42..],
    NOT_REQUIRED, text: Text [FIX40..],
    NOT_REQUIRED, encoded_text_len: EncodedTextLen [FIX42..],
    NOT_REQUIRED, encoded_text: EncodedText [FIX42..],
});

define_fixt_message!(SequenceReset: ADMIN b"4" => {
    NOT_REQUIRED, gap_fill_flag: GapFillFlag [FIX40..],
    REQUIRED, new_seq_no: NewSeqNo [FIX40..],
});

define_fixt_message!(Logout: ADMIN b"5" => {
    NOT_REQUIRED, session_status: SessionStatus [FIX50SP1..],
    NOT_REQUIRED, text: Text [FIX40..],
    NOT_REQUIRED, encoded_text_len: EncodedTextLen [FIX42..],
    NOT_REQUIRED, encoded_text: EncodedText [FIX42..],
});

//Other Messages

define_fixt_message!(Email: b"C" => {
    REQUIRED, email_thread_id: EmailThreadID [FIX41..],
    REQUIRED, email_type: EmailType [FIX40..],
    NOT_REQUIRED, orig_time: OrigTime [FIX40..],
    NOT_REQUIRED, related_sym: RelatedSym [FIX40..FIX41],
    REQUIRED, subject: Subject [FIX41..],
    NOT_REQUIRED, encoded_subject_len: EncodedSubjectLen [FIX42..],
    NOT_REQUIRED, encoded_subject: EncodedSubject [FIX42..],
    NOT_REQUIRED, no_routing_ids: NoRoutingIDs [FIX42..],
    NOT_REQUIRED, no_related_sym: NoRelatedSym [FIX41..],
    NOT_REQUIRED, no_underlyings: NoUnderlyings [FIX44..],
    NOT_REQUIRED, no_legs: NoLegs [FIX44..],
    NOT_REQUIRED, order_id: OrderID [FIX40..],
    NOT_REQUIRED, cl_ord_id: ClOrdID [FIX40..],
    REQUIRED, no_lines_of_text: NoLinesOfText [FIX40..],
    NOT_REQUIRED, raw_data_length: RawDataLength [FIX40..],
    NOT_REQUIRED, raw_data: RawData [FIX40..],
});

define_fixt_message!(BusinessMessageReject: b"j" => {
    NOT_REQUIRED, ref_seq_num: RefSeqNum [FIX42..],
    REQUIRED, ref_msg_type: RefMsgType [FIX42..],
    NOT_REQUIRED, ref_appl_ver_id: RefApplVerID [FIX50SP1..],
    NOT_REQUIRED, ref_appl_ext_id: RefApplExtID [FIX50SP1..],
    NOT_REQUIRED, ref_cstm_appl_ver_id: RefCstmApplVerID [FIX50SP1..],
    NOT_REQUIRED, business_reject_ref_id: BusinessRejectRefID [FIX42..],
    REQUIRED, business_reject_reason: BusinessRejectReason [FIX42..],
    NOT_REQUIRED, text: Text [FIX42..],
    NOT_REQUIRED, encoded_text_len: EncodedTextLen [FIX42..],
    NOT_REQUIRED, encoded_text: EncodedText [FIX42..],
});

define_fixt_message!(NewOrderSingle: b"D" => { //TODO: All version info for this message is wrong.
    REQUIRED, cl_ord_id: ClOrdID [FIX40..],
    /*NOT_REQUIRED, secondary_cl_ord_id: SecondaryClOrdID,
    NOT_REQUIRED, cl_ord_link_id: ClOrdLinkID,
    NOT_REQUIRED, parties: NoParties,
    NOT_REQUIRED, trade_origination_date: TradeOriginationDate,
    NOT_REQUIRED, trade_date: TradeDate,*/
    NOT_REQUIRED, account: Account [FIX40..],
    /*NOT_REQUIRED, acct_id_source: AcctIDSource,
    NOT_REQUIRED, account_type: AccountType,
    NOT_REQUIRED, day_booking_inst: DayBookingInst,
    NOT_REQUIRED, booking_unit: BookingUnit,
    NOT_REQUIRED, prealloc_method: PreallocMethod,
    NOT_REQUIRED, alloc_id: AllocID,
    NOT_REQUIRED, pre_alloc_grp: NoPreAllocGrp,*/
    NOT_REQUIRED, settl_type: SettlType [FIX40..],
    NOT_REQUIRED, settl_date: SettlDate [FIX40..],
    /*NOT_REQUIRED, cash_margin: CashMargin,
    NOT_REQUIRED, clearing_free_indicator: ClearingFreeIndicator,*/
    NOT_REQUIRED, handl_inst: HandlInst [FIX40..],
    /*NOT_REQUIRED, exec_inst: ExecInst,*/
    NOT_REQUIRED, min_qty: MinQty [FIX40..],
    /*NOT_REQUIRED, match_increment: MatchIncrement,
    NOT_REQUIRED, max_price_levels: MaxPriceLevels,
    NOT_REQUIRED, display_instruction: NoDisplayInstruction,*/
    NOT_REQUIRED, max_floor: MaxFloor [FIX40..],
    /*NOT_REQUIRED, ex_destination: ExDestination,
    NOT_REQUIRED, ex_destination_id_source: ExDestinationIDSource,
    NOT_REQUIRED, trdg_ses_grp: NoTrdgSesGrp,
    NOT_REQUIRED, process_code: ProcessCode,
    REQUIRED, instrument: NoInstrument,*/
        REQUIRED, symbol: Symbol [FIX40..], //TODO: Part of the Instrument block.
        REQUIRED, security_id: SecurityID [FIX40..], //TODO: Part of the Instrument block.
        REQUIRED, security_id_source: SecurityIDSource [FIX40..], //TODO: Part of the Instrument block.
    /*NOT_REQUIRED, financing_details: NoFinancingDetails,
    NOT_REQUIRED, und_instrmt_grp: NoUndInstrmtGrp,
    NOT_REQUIRED, prev_close_px: PrevClosePx,*/
    REQUIRED, side: SideField [FIX40..],
    /*NOT_REQUIRED, locate_reqd: LocateReqd,*/
    REQUIRED, transact_time: TransactTime [FIX40..],
    /*NOT_REQUIRED, stipulations: NoStipulations,
    NOT_REQUIRED, qty_type: QtyType,*/
    REQUIRED, order_qty: OrderQty [FIX40..], //TODO: One and only one of OrderQty, CashOrderQty or OrderPrecent should be specified.
    /*NOT_REQUIRED, cash_order_qty: CashOrderQty,
    NOT_REQUIRED, order_precent: OrderPercent,
    NOT_REQUIRED, rounding_direction: RoundingDirection,
    NOT_REQUIRED, rounding_modulus: RoundingModulus,*/
    REQUIRED, ord_type: OrdType [FIX40..],
    /*NOT_REQUIRED, price_type: PriceType,*/
    NOT_REQUIRED, price: Price [FIX40..],
    /*NOT_REQUIRED, price_protection_scope: PriceProtectionScope,
    NOT_REQUIRED, stop_px: StopPx,
    NOT_REQUIRED, triggering_instruction: NoTriggeringInstruction,
    NOT_REQUIRED, spread_or_benchmark_curve_data: NoSpreadOrBenchmarkCurveData,
    NOT_REQUIRED, yield_data: NoYieldData,*/
    NOT_REQUIRED, currency: Currency [FIX40..],
    /*NOT_REQUIRED, compliance_id: ComplianceID,
    NOT_REQUIRED, solicited_flag: SolicitedFlag,
    NOT_REQUIRED, ioi_id: IOIID,
    NOT_REQUIRED, quote_id: QuoteID,*/
    NOT_REQUIRED, time_in_force: TimeInForce [FIX40..],
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
