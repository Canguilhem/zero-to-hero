#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Order, to_binary};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, AllPollsResponse, PollResponse, VoteResponse,ConfigResponse,AllVotesForUser};

use crate::state::{Config,CONFIG, Poll,POLLS, Ballot,BALLOTS};


const CONTRACT_NAME: &str = "crates.io:cw-starter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
 

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage,CONTRACT_NAME,CONTRACT_VERSION);

    let admin = msg.admin.unwrap_or(info.sender.to_string());
    let validated_admin = deps.api.addr_validate(&admin)?;
    let config = Config {
        admin: validated_admin.clone(),
    };
    CONFIG.save(deps.storage,&config)?;

    // no trailing ';' imply return

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", validated_admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreatePoll{
            poll_id,
            question,
            options,
        }=> execute_create_poll(deps, env, info, poll_id, question, options),
        ExecuteMsg::Vote { poll_id,vote}=> execute_vote(deps, env, info, poll_id, vote),
        ExecuteMsg::DeletePoll{poll_id}=> execute_delete_poll(deps, env, info, poll_id),
        ExecuteMsg::RevokeVote{poll_id}=> execute_revoke_vote(deps, env, info, poll_id),

    }
    
}

fn execute_create_poll(deps:DepsMut,_env:Env,info:MessageInfo,poll_id:String,question:String, options:Vec<String>) -> Result<Response,ContractError>{
    if options.len() > 10 {
        return Err(ContractError::TooManyOptions {})
    }

    let mut opts: Vec<(String,u64)>= vec![];

    for option in options {
        opts.push((option,0))
    }

    let poll = Poll {
        creator:info.sender.clone(),
        question,
        options:opts
    };

    POLLS.save(deps.storage, poll_id,&poll)?;

    // TODO add some tests to check if ownership is respected here
    Ok(Response::new()
        .add_attribute("action", "create_poll")
        .add_attribute("creator", info.sender))
}

fn execute_vote(deps:DepsMut,_env:Env,info:MessageInfo,poll_id:String,vote:String) -> Result<Response,ContractError>{
    // check if poll exists
    let poll= POLLS.may_load(deps.storage, poll_id.clone())?;
    
    // check if user already voted and update accordingly
    match poll {
        Some(mut poll) => {
            BALLOTS.update(
                deps.storage,
                (info.sender.clone(), poll_id.clone()),
                |ballot| -> StdResult<Ballot> {
                    match ballot {
                         // user has already voted
                        Some(ballot) => {
                            let position_of_old_vote = poll
                                .options
                                .iter()
                                .position(|option| option.0 == ballot.option)
                                .unwrap();
                            
                            poll.options[position_of_old_vote].1 -= 1;
                            
                            Ok(Ballot { option: vote.clone() })
                        }
                        // user hasnt voted yet
                        None => {
                            
                            Ok(Ballot { option: vote.clone() })
                        }
                    }
                },
            )?;
            let position = poll
            .options
            .iter()
            .position(|option| option.0 == vote);
        if position.is_none() {
            return Err(ContractError::UnknownOption {});
        }
        let position = position.unwrap();
        poll.options[position].1 += 1;

        POLLS.save(deps.storage, poll_id, &poll)?;
        Ok(Response::new()
            .add_attribute("action", "vote")
            .add_attribute("voter", info.sender)
            .add_attribute("vote", vote)
        )
    },
        None => Err(ContractError::PollNotFound {}), // The poll does not exist so we just error
    }  
}

// TODO
fn execute_delete_poll(deps:DepsMut,_env:Env,info:MessageInfo,poll_id:String) -> Result<Response,ContractError>{
    unimplemented!()
}

// TODO
fn execute_revoke_vote(deps:DepsMut,_env:Env,info:MessageInfo,poll_id:String) -> Result<Response,ContractError>{
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllPolls {} => query_all_polls(deps,env),
        QueryMsg::Poll { poll_id } => query_poll(deps, env, poll_id),
        QueryMsg::Vote { address, poll_id } => query_vote(deps, env, address, poll_id),
        QueryMsg::Config { } => query_config(deps,env),
        QueryMsg::AllVotesForUser { address } => query_all_votes_for_user(deps, env, address),
    }
}

fn query_all_polls(deps:Deps,_env:Env) -> StdResult<Binary> {
    let polls = POLLS  
    // none here means no min or max value for range
        .range(deps.storage, None,None,Order::Ascending)
        .map(|p| Ok(p?.1))
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&AllPollsResponse { polls })
}

fn query_poll(deps:Deps,_env:Env, poll_id:String) -> StdResult<Binary> {
    let poll = POLLS.may_load(deps.storage, poll_id)?;
    to_binary(&PollResponse { poll })
}

fn query_vote(deps:Deps,_env:Env, address:String,poll_id:String) -> StdResult<Binary> {
    let validated_address = deps.api.addr_validate(&address).unwrap();
    let vote = BALLOTS.may_load(deps.storage, (validated_address, poll_id))?;
    to_binary(&VoteResponse { vote })
}

//    FIXME
fn query_config(deps:Deps,_env:Env) -> StdResult<Binary> {
    let config = CONFIG.may_load(deps.storage)?;
    // to_binary(&ConfigResponse { config })
    unimplemented!()
}

fn query_all_votes_for_user(deps:Deps,_env:Env, address:String) -> StdResult<Binary> {
    unimplemented!()
}


#[cfg(test)]
mod tests {
    use cosmwasm_std::attr;
    use cosmwasm_std::testing::{mock_dependencies,mock_env,mock_info};
    use crate::contract::{instantiate, execute};
    use crate::msg::{InstantiateMsg, ExecuteMsg};
    use crate::error::ContractError;

    pub const ADDR1: & str= "addr1";
    pub const ADDR2: & str= "addr2";

    #[test]
    fn test_instantiate(){
        // instantiating mocks
        let mut deps = mock_dependencies();
        let env= mock_env();
        let info= mock_info(ADDR1, &vec![]);

        let msg = InstantiateMsg {admin:None};
        let res = instantiate(deps.as_mut(), env,info,msg).unwrap();

        assert_eq!(res.attributes, vec![attr("action","instantiate"), attr("admin",ADDR1)])
    }

    #[test]
    fn test_instantiate_with_admin(){
        let mut deps = mock_dependencies();
        let env= mock_env();
        let info= mock_info(ADDR1, &vec![]);

        let msg = InstantiateMsg {admin:Some(ADDR2.to_string())};
        let res = instantiate(deps.as_mut(), env,info,msg).unwrap();

        assert_eq!(res.attributes, vec![attr("action","instantiate"), attr("admin",ADDR2)])


    }

    #[test]
    fn test_execute_create_poll_valid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &vec![]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CreatePoll {
            poll_id:"some_id".to_string(),
            question:"What's your favorite Cosmos coin".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ]
        };

        let res = execute(deps.as_mut(), env,info,msg).unwrap();

        assert_eq!(res.attributes, vec![attr("action","create_poll"), attr("creator",ADDR1)])
    }

    #[test]
    fn test_execute_create_poll_invalid() {
        let mut deps = mock_dependencies();
        let env = mock_env();   
        let info = mock_info(ADDR1, &vec![]);   
        // Instantiate the contract 
        let msg = InstantiateMsg { admin: None };   
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap(); 

        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id".to_string(),
            question: "What's your favourite number?".to_string(),
            options: vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
                "10".to_string(),
                "11".to_string(),
            ],
        };

        // Unwrap error to assert failure
        let _err = execute(deps.as_mut(), env, info, msg).unwrap_err();

    }

    #[test]
    fn test_execute_vote_valid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &vec![]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create the poll
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Vote{
            poll_id: "some_id".to_string(),
            vote: "Juno".to_string(),
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();


        assert_eq!(res.attributes, vec![attr("action","vote"), attr("voter",ADDR1), attr("vote", "Juno".to_string())])
    }

    #[test]
    fn test_execute_create_vote_invalid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &vec![]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Vote {
            poll_id: "some_id".to_string(),
            vote: "Juno".to_string(),
        };
        // Unwrap to assert error
        let _err = execute(deps.as_mut(), env.clone(), info.clone(), msg);

        match _err {
            Err(ContractError::PollNotFound {}) => (),
            Err(e) => panic!("Unexpected error: {:?}", e),
            _ => panic!("Must return error"),
        }

         // Create the poll
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Vote{
            poll_id: "some_id".to_string(),
            vote: "AXELAR".to_string()
        };

        let _err = execute(deps.as_mut(), env, info, msg);


        match _err {
            Err(ContractError::UnknownOption {}) => (),
            Err(e) => panic!("Unexpected error: {:?}", e),
            _ => panic!("Must return error"),
        }

    }
}
