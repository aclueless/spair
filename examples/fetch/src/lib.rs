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
            .callback_arg_mut(|state: &mut Self, result| match result {
                Ok(branch) => state.set_data(branch),
                Err(err) => state.fetch_error(err),
            });
        fetch_repo_metadata().spawn_local_with(callback);
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

impl spair::Component for State {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .static_text("You are running `examples\\fetch`")
            .line_break()
            .match_if(|mi| match self.branch.as_ref() {
                Some(branch) => spair::set_arm!(mi)
                    .rfn(|nodes| render_branch(branch, nodes))
                    .button(|b| {
                        b.static_attributes()
                            .on_click(comp.handler_mut(State::reset))
                            .static_nodes()
                            .static_text("Reset");
                    })
                    .done(),
                None => spair::set_arm!(mi)
                    .button(|b| {
                        b.static_attributes()
                            .on_click(comp.handler_mut(State::start_fetching))
                            .static_nodes()
                            .static_text("Click to fetch wasm-bindgen latest commit info");
                    })
                    .done(),
            })
            .p(|p| p.update_text(&self.message).done());
    }
}

fn render_branch(branch: &Branch, nodes: spair::Nodes<State>) {
    nodes
        .p(|p| {
            p.static_text("The latest commit to the wasm-bindgen ")
                .update_text(&branch.name)
                .static_text(" branch is:");
        })
        .rfn(|nodes| render_commit(&branch.commit, nodes));
}

fn render_commit(commit: &Commit, nodes: spair::Nodes<State>) {
    nodes.p(|p| {
        p.update_text(&commit.sha)
            .static_text(", authored by ")
            .update_text(&commit.commit.author.name)
            .static_text(" (")
            .update_text(&commit.commit.author.email)
            .static_text(")");
    });
}

impl spair::Application for State {
    fn init(comp: &spair::Comp<Self>) -> Self {
        Self {
            comp: comp.clone(),
            branch: None,
            message: "Wait for your click".to_string(),
        }
    }
}

#[wasm_bindgen(start)]
pub fn start_fetch_example() {
    State::mount_to_element_id("root")
}
