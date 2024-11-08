use cosmwasm_std::{ Deps, StdError, StdResult,Env};
use crate::{
    msg::{ProposalResponse,VoteInfo,VoteResponse},
    state::{BALLOTS,PROPOSALS,Votes}
};
pub fn query_proposal(deps: Deps, env: Env, id: u64) -> StdResult<ProposalResponse> {
    let prop = PROPOSALS.load(deps.storage, id)?;
    let status = prop.current_status(&env.block);
    Ok(ProposalResponse {
        id,
        title: prop.title,
        description: prop.description,
        // msgs: prop.msgs,
        status,
        expires: prop.expires,
        // deposit: prop.deposit,
        proposer: prop.proposer,
        // threshold,
    })
}

/*
fn query_vote(deps: Deps, voter: String) -> StdResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter)?;
    let ballot = BALLOTS.may_load(deps.storage, &voter)?;

    let vote = ballot.map(|b| VoteInfo {
        voter: voter.into(),
        vote: b,
    });
    Ok(VoteResponse { vote })
}
*/
pub fn query_vote(deps: Deps, voter: String, proposal_id: u64) -> StdResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter)?;
    let ballot = BALLOTS.may_load(deps.storage, (proposal_id, &voter))?;

    let vote = match ballot {
        Some(b) => VoteInfo {
            voter: voter.into(),
            vote: b,
        },
        None => {
            // Gestisci il caso in cui il voto non è stato trovato
            return Err(StdError::generic_err("Vote not found"));
        }
    };

    Ok(VoteResponse { vote })
}

pub fn query_proposal_response(deps: Deps, proposal_id: u64) -> StdResult<Votes> {
    let prop = PROPOSALS.load(deps.storage, proposal_id)?;
    let votes = prop.votes;
    Ok(Votes {
        a: votes.a,
        b: votes.b,
        c: votes.c,
        d: votes.d,
    })
}