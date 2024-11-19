use crate::{
    msg::{
        ProposalIdsWithTitlesResponse, ProposalResponse, ProposalResult, ProposalsByProposerResponse, StatusResponse, VotersResponse
    },
    state::{
        VoterStatus, Votes, PROPOSALS, STATUS, VOTERS
    },
};
use cosmwasm_std::{Addr, Deps, Env, StdResult};

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
pub fn query_proposals_by_proposer(
    deps: Deps,
    proposer: Addr,
) -> StdResult<ProposalsByProposerResponse> {
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

pub fn query_proposal_running_response(deps: Deps, proposal_id: u64) -> StdResult<Votes> {
    let prop = PROPOSALS.load(deps.storage, proposal_id)?;
    let votes_response = Votes {
        counts: prop.votes.counts.clone(),
    };
    Ok(votes_response)
}

pub fn query_voters(deps: Deps, _env: Env, proposal_id: u64) -> StdResult<VotersResponse> {
    // Filtra i votanti per la proposta specifica
    let mut allowed_voters = Vec::new();
    let mut pending_voters = Vec::new();
    let mut has_voted_voters = Vec::new();

    VOTERS
        .prefix(proposal_id)
        .range(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .for_each(|result| {
            if let Ok((addr, status)) = result {
                match status {
                    VoterStatus::CanVote => allowed_voters.push(addr.clone()),
                    VoterStatus::NotAllowed => pending_voters.push(addr.clone()),
                    VoterStatus::HasVoted => has_voted_voters.push(addr.clone()),
                }
            }
        });

    // Costruisce la risposta
    let response = VotersResponse {
        allowed_voters,
        pending_voters,
        has_voted_voters,
    };

    Ok(response)
}

pub fn query_status(deps: Deps) -> StdResult<StatusResponse> {
    let state = STATUS.load(deps.storage)?;
    let response = StatusResponse {
        admin: state.admin.to_string(),
        commissions: state
            .commissions
            .iter()
            .map(|coin| format!("{} {}", coin.amount, coin.denom))
            .collect(),
        voting_fee: state.voting_fee,
    };
    Ok(response)
}