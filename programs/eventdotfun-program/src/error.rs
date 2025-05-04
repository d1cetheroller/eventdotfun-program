use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Authority")]
    InvalidAuthority,

    #[msg("Invalid Sales Type, should be 1 or 2")]
    InvalidSalesType,

    #[msg("Invalid Timestamp")]
    InvalidTimestamp,

    #[msg("Invalid Exponent, should be in between 1 - 3")]
    InvalidExponent,

    #[msg("Invalid Price")]
    InvalidPrice,

    #[msg("Invalid Ticket Configuration")]
    InvalidTicketConfiguration,

    #[msg("Max Ticket Reached")]
    MaxTicketReached,

    #[msg("Curve Not Started Yet")]
    CurveNotStarted,

    #[msg("Curve Still On Progess")]
    CurveStillOnProgress,

    #[msg("Curve Ended")]
    CurveEnded,

    #[msg("Curve Reaches Threshold")]
    CurveReachesThreshold,

    #[msg("Curve Still Below Threshold")]
    CurveStillBelowThreshold,

    #[msg("Refund Not Opened")]
    RefundNotOpened,
}
