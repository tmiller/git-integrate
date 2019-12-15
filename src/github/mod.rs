use graphql_client::{GraphQLQuery, Response};

use super::git_extras::Repo;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/github/schema.json",
    query_path = "src/github/queries.graphql",
    response_derives = "Debug,Clone"
)]
pub struct LabelBranches;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/github/schema.json",
    query_path = "src/github/queries.graphql",
    response_derives = "Debug,Clone"
)]
pub struct MilestoneBranches;

pub fn branches_by_pr_label(
    token: String,
    repo: Repo,
    label: String,
) -> Result<Vec<String>, reqwest::Error> {
    let q = LabelBranches::build_query(label_branches::Variables {
        owner: repo.owner,
        name: repo.name,
        label: label,
    });

    let client = reqwest::Client::new();

    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&q)
        .send()?;

    let response: Response<label_branches::ResponseData> = res.json()?;

    Ok(response
        .data
        .and_then(|x| x.repository)
        .and_then(|x| x.pull_requests.nodes)
        .unwrap_or(vec![])
        .iter()
        .cloned()
        .filter_map(|x| x.map(|y| y.head_ref_name))
        .collect())
}

pub fn branches_by_milestone(
    token: String,
    repo: Repo,
    milestone: i64,
) -> Result<Vec<String>, reqwest::Error> {
    let q = MilestoneBranches::build_query(milestone_branches::Variables {
        owner: repo.owner,
        name: repo.name,
        milestone: milestone,
    });

    let client = reqwest::Client::new();

    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(token)
        .json(&q)
        .send()?;

    let response: Response<milestone_branches::ResponseData> = res.json()?;

    Ok(response
        .data
        .and_then(|x| x.repository)
        .and_then(|x| x.milestone)
        .and_then(|x| x.pull_requests.nodes)
        .unwrap_or(vec![])
        .iter()
        .cloned()
        .filter_map(|x| x.map(|y| y.head_ref_name))
        .collect())
}
