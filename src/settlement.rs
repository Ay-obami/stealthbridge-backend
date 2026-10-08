//! Pure settlement state machine; no chain or payout side effects.
//! A payout is NOT complete just because an on-chain payment finalized.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettlementState {
    Draft, Quoted, Authorized, Submitted,
    ChainFinalized, PayoutPending, PayoutCompleted,
    Expired, Rejected, ChainFailed, PayoutFailed,
    RefundPending, Refunded, ManualReview,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransitionError {
    Invalid { from: SettlementState, to: SettlementState },
}

pub fn transition(from: SettlementState, to: SettlementState) -> Result<SettlementState, TransitionError> {
    use SettlementState::*;
    let valid = matches!((from,to),
        (Draft,Quoted) | (Draft,Rejected) |
        (Quoted,Authorized) | (Quoted,Expired) | (Quoted,Rejected) |
        (Authorized,Submitted) | (Authorized,Rejected) |
        (Submitted,ChainFinalized) | (Submitted,ChainFailed) | (Submitted,ManualReview) |
        (ChainFinalized,PayoutPending) | (ChainFinalized,ManualReview) |
        (PayoutPending,PayoutCompleted) | (PayoutPending,PayoutFailed) | (PayoutPending,ManualReview) |
        (PayoutFailed,PayoutPending) | (PayoutFailed,RefundPending) | (PayoutFailed,ManualReview) |
        (ChainFailed,RefundPending) | (ChainFailed,ManualReview) |
        (RefundPending,Refunded) | (RefundPending,ManualReview) |
        (ManualReview,PayoutPending) | (ManualReview,RefundPending)
    );
    if valid {Ok(to)} else {Err(TransitionError::Invalid {from,to})}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn happy_path_requires_separate_payout_confirmation() {
        use SettlementState::*;
        let mut state=Draft;
        for next in [Quoted,Authorized,Submitted,ChainFinalized,PayoutPending,PayoutCompleted] {
            state=transition(state,next).expect("valid transition");
        }
        assert_eq!(state,PayoutCompleted);
    }
    #[test]
    fn chain_finality_cannot_be_called_payout() {
        use SettlementState::*;
        assert_eq!(transition(ChainFinalized,PayoutCompleted),
        Err(TransitionError::Invalid {from:ChainFinalized,to:PayoutCompleted}));
    }
    #[test]
    fn finalized_transfer_cannot_be_cancelled() {
        use SettlementState::*;
        assert!(transition(ChainFinalized,Rejected).is_err());
    }
    #[test]
    fn payout_failure_requires_recovery() {
        use SettlementState::*;
        assert_eq!(transition(PayoutFailed,RefundPending),Ok(RefundPending));
        assert_eq!(transition(RefundPending,Refunded),Ok(Refunded));
    }
}
