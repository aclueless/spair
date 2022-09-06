// Copied from https://github.com/rustwasm/wasm-bindgen/blob/master/examples/fetch/src/lib.rs
// and modified to work with spair

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

    fn start_fetching(&mut self) -> spair::Command<Self> {
        self.message = "Clicked! Please wait for a moment".to_string();

        spair::http::Request::get(
            "https://api.github.com/repos/rustwasm/wasm-bindgen/branches/master",
        )
        .header("Accept", "application/vnd.github.v3+json")
        .text_mode()
        // .body().json(data) <== if you are `spair::Request::post`ing something
        .response()
        // Please note that you must enable `features = ["fetch-json"]`
        .json(State::set_data, State::fetch_error)
    }

    fn fetch_error(&mut self, e: spair::FetchError) {
        self.message = e.to_string();
    }
}

impl spair::Component for State {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element
            .rstatic("You are running `examples\\fetch`")
            .line_break()
            .match_if(|mi| match self.branch.as_ref() {
                Some(branch) => spair::set_arm!(mi)
                    .rupdate(branch)
                    .button(|b| {
                        b.static_attributes()
                            .on_click(comp.handler_mut(State::reset))
                            .static_nodes()
                            .rstatic("Reset");
                    })
                    .done(),
                None => spair::set_arm!(mi)
                    .button(|b| {
                        b.static_attributes()
                            .on_click(comp.handler_mut(State::start_fetching))
                            .static_nodes()
                            .rstatic("Click to fetch wasm-bindgen latest commit info");
                    })
                    .done(),
            })
            .p(|p| p.rupdate(&self.message).done());
    }
}

impl spair::Render<State> for &Branch {
    fn render(self, nodes: spair::Nodes<State>) {
        nodes
            .p(|p| {
                p.rstatic("The latest commit to the wasm-bindgen ")
                    .rupdate(&self.name)
                    .rstatic(" branch is:");
            })
            .rupdate(&self.commit);
    }
}

impl spair::Render<State> for &Commit {
    fn render(self, nodes: spair::Nodes<State>) {
        nodes.p(|p| {
            p.rupdate(&self.sha)
                .rstatic(", authored by ")
                .rupdate(&self.commit.author.name)
                .rstatic(" (")
                .rupdate(&self.commit.author.email)
                .rstatic(")");
        });
    }
}

impl spair::Application for State {
    fn init(_: &spair::Comp<Self>) -> Self {
        Self {
            branch: None,
            message: "Wait for your click".to_string(),
        }
    }
}

#[wasm_bindgen(start)]
pub fn start_fetch_example() {
    State::mount_to("root")
}
