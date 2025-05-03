use gloo_net::http;
use serde::{Deserialize, Serialize};
use spair::prelude::*;

/// A struct to hold some data from the github Branch API.
///
/// Note how we don't have to define every member -- serde will ignore extra
/// data when deserializing
#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub commit: Commit,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub commit: CommitDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitDetails {
    pub author: Signature,
    pub committer: Signature,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Signature {
    pub name: String,
    pub email: String,
}

struct State {
    comp: spair::Comp<Self>,
    branch: Option<Branch>,
    message: String,
}

impl State {
    fn set_data(&mut self, branch: Branch) {
        self.branch = Some(branch);
        self.message = "".to_string();
    }

    fn reset(&mut self) {
        self.branch = None;
        self.message = "Wait for your click".to_string();
    }

    fn start_fetching(&mut self) {
        self.message = "Clicked! Please wait for a moment".to_string();
        let callback = self
            .comp
            .callback_arg(|state: &mut Self, result| match result {
                Ok(branch) => state.set_data(branch),
                Err(err) => state.fetch_error(err),
            });
        fetch_repo_metadata().spawn_local(callback);
    }

    fn fetch_error(&mut self, e: gloo_net::Error) {
        self.message = e.to_string();
    }
}

async fn fetch_repo_metadata() -> Result<Branch, gloo_net::Error> {
    http::Request::get("https://api.github.com/repos/rustwasm/wasm-bindgen/branches/master")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .json()
        .await
}

#[component_for]
impl State {
    fn create(_ccontext: &Context<State>) {}
    fn update(ucontext: &Context<State>) {}
    fn view() {
        div(
            replace_at_element_id = "root",
            match ucontext.state.branch.as_ref() {
                Some(branch) => div(
                    v.BranchView(branch),
                    button(
                        on_click = ucontext.comp.callback_arg(|state, _| state.reset()),
                        text("Reset"),
                    ),
                ),
                None => button(
                    on_click = ucontext
                        .comp
                        .callback_arg(|state, _| state.start_fetching()),
                    text("Click to fetch the latest commit info of the wasm-bindgen repo"),
                ),
            },
            p(text(&ucontext.state.message)),
        )
    }
}

#[new_view]
impl BranchView {
    fn create(branch: &Branch) {}
    fn update() {}
    fn view() {
        p(text(
            "The latest commit to the wasm-bindgen ",
            &branch.name,
            " branch is: ",
            &branch.commit.sha,
            ", authored by ",
            &branch.commit.commit.author.name,
            " (",
            &branch.commit.commit.author.email,
            ")",
        ))
    }
}

pub fn main() {
    spair::start_app(|comp| State {
        comp: comp.clone(),
        branch: None,
        message: "Wait for your click".to_string(),
    });
}
