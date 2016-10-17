use std::any::Any;
use std::collections::{HashMap,HashSet};
use field::{Action,FieldType,Field,StringFieldType,DataFieldType,NoneFieldType,RepeatingGroupFieldType};
use message::{REQUIRED,NOT_REQUIRED,Meta,Message};

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
define_field!(
    Account: StringFieldType = b"1",
    ClOrdID: StringFieldType = b"11",
    Currency: StringFieldType = b"15", //Currency
    HandInst: StringFieldType = b"21", //Char, TODO: limited choices.
    SecurityIDSource: StringFieldType = b"22", //TODO: Limited choices.
    MsgSeqNum: StringFieldType = b"34", //TODO: Special field probably should be built into the parser.
    OrderQty: StringFieldType = b"38", //Qty
    OrdType: StringFieldType = b"40", //Char, TODO: limited choices.
    Price: StringFieldType = b"44", //Price
    SecurityID: StringFieldType = b"48",
    SenderCompID: StringFieldType = b"49",
    SendingTime: StringFieldType = b"52", //UTCTimestamp
    Side: StringFieldType = b"54", //Char, TODO: limited choices.
    Symbol: StringFieldType = b"55",
    TargetCompID: StringFieldType = b"56",
    TimeInForce: StringFieldType = b"59", //Char, TODO: limited choices.
    TransactTime: StringFieldType = b"60", //UTCTimestamp
    SettlType: StringFieldType = b"63", //TODO: Limited choices.
    SettlDate: StringFieldType = b"64", //LocalMktDate
    NoOrders: RepeatingGroupFieldType<Order> = b"73",
    NoAllocs: RepeatingGroupFieldType<Alloc> = b"78",
    AllocAccount: StringFieldType = b"79",
    RawDataLength: NoneFieldType = b"95" => Action::PrepareForBytes{ bytes_tag: RawData::tag() },
    RawData: DataFieldType = b"96" => Action::ConfirmPreviousTag{ previous_tag: RawDataLength::tag() },
    EncryptMethod: StringFieldType = b"98",
    HeartBtInt: StringFieldType = b"108",
    MinQty: StringFieldType = b"110", //Qty
    MaxFloor: StringFieldType = b"111", //Qty
    BidSize: StringFieldType = b"134", //Qty
    CashOrderQty: StringFieldType = b"152", //Qty
    RefMsgType: StringFieldType = b"372",
    NoMsgTypeGrp: RepeatingGroupFieldType<MsgTypeGrp> = b"384",
    MsgDirection: StringFieldType = b"385", //Char
    RefApplVerID: StringFieldType = b"1130",
    RefCstmApplVerID: StringFieldType = b"1131",
    RefApplExtID: StringFieldType = b"1406", //int
    DefaultVerIndicator: StringFieldType = b"1410", //bool
    NoRateSources: RepeatingGroupFieldType<RateSource> = b"1445",
    RateSourceField: StringFieldType = b"1446", //int
    RateSourceType: StringFieldType = b"1447", //int
    ReferencePage: StringFieldType = b"1448",
);

//TODO: all of the following messages are incomplete in one way or another right now.

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

define_message!(Logon {
    REQUIRED, encrypt_method: EncryptMethod,
    REQUIRED, heart_bt_int: HeartBtInt,
    REQUIRED, msg_seq_num: MsgSeqNum,
    NOT_REQUIRED, sending_time: SendingTime,
    NOT_REQUIRED, sender_comp_id: SenderCompID,
    NOT_REQUIRED, target_comp_id: TargetCompID,
    NOT_REQUIRED, raw_data_length: RawDataLength,
    NOT_REQUIRED, raw_data: RawData,
    NOT_REQUIRED, msg_type_grp: NoMsgTypeGrp,
});

//TODO: Support embedding redundant blocks into messages. ie. OrderQtyData.

define_message!(NewOrderSingle {
    REQUIRED, cl_ord_id: ClOrdID,
    REQUIRED, sender_comp_id: SenderCompID, //TODO: Part of the FIXT standard header.
    REQUIRED, target_comp_id: TargetCompID, //TODO: Part of the FIXT standard header.
    REQUIRED, msg_seq_num: MsgSeqNum, //TODO: Part of the FIXT standard header.
    REQUIRED, sending_time: SendingTime, //TODO: Part of the FIXT standard header.
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
