use crate::{
    msg::{ProposalIdsWithTitlesResponse, ProposalResponse, ProposalResult, ProposalsByProposerResponse, VoteInfo, VoteResponse},
    state::{Votes, BALLOTS, PROPOSALS},
};
use cosmwasm_std::{Addr, Deps, Env, StdError, StdResult};

pub fn query_all_proposal_ids_with_titles(deps: Deps) -> StdResult<ProposalIdsWithTitlesResponse> {
    let proposals: Vec<(u64, String)> = PROPOSALS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .map(|item| {
            item.map(|(id, proposal)| (id, proposal.title.clone())) // Mappa ID e titolo
        })
        .collect::<StdResult<Vec<(u64, String)>>>()?;

    let response = ProposalIdsWithTitlesResponse { proposals };
    Ok(response)
}
pub fn query_proposals_by_proposer(deps: Deps, proposer: Addr) -> StdResult<ProposalsByProposerResponse> {
    let mut proposals: Vec<(u64, String)> = Vec::new();

    PROPOSALS
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .for_each(|item| {
            if let Ok((id, proposal)) = item {
                if proposal.proposer == proposer {
                    proposals.push((id, proposal.title.clone()));
                }
            }
        });

    let response = ProposalsByProposerResponse { proposals };
    Ok(response)
}

pub fn query_proposal(deps: Deps, env: Env, id: u64) -> StdResult<ProposalResponse> {
    let prop = PROPOSALS.load(deps.storage, id)?;
    let status = prop.current_status(&env.block);
    Ok(ProposalResponse {
        id,
        title: prop.title,
        description: prop.description,
        status,
        expires: prop.expires,
        proposer: prop.proposer,
        options: prop.option,
    })
}

pub fn query_proposal_result(deps: Deps, proposal_id: u64) -> StdResult<ProposalResult> {
    let proposal = PROPOSALS.load(deps.storage, proposal_id)?;
    let result = ProposalResult {
        title: proposal.title,
        description: proposal.description,
        winner: proposal.winner,
    };

    Ok(result)
}

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

pub fn query_proposal_running_response(deps: Deps, proposal_id: u64) -> StdResult<Votes> {
    let prop = PROPOSALS.load(deps.storage, proposal_id)?;
    let votes = prop.votes;
    Ok(Votes {
        a: votes.a,
        b: votes.b,
        c: votes.c,
        d: votes.d,
    })
}
