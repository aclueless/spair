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
}

impl State {
    fn set_data(&mut self, branch: Branch) {
        self.branch = Some(branch);
    }

    fn reset(&mut self) {
        self.branch = None;
    }

    fn start_fetching(&mut self) -> spair::Checklist<Self> {
        let mut c = spair::Checklist::skip_fn_render();
        let request = spair::Request::get(
            "https://api.github.com/repos/rustwasm/wasm-bindgen/branches/master",
        )
        .header("Accept", "application/vnd.github.v3+json")
        .body(None)
        .unwrap();

        c.fetch_json_ok_error(request, None, State::set_data, State::fetch_error);
        c
    }

    fn fetch_error(&mut self, e: spair::FetchError) {
        log::error!("{}", e);
    }
}

impl spair::Component for State {
    type Routes = ();
    fn render(&self, c: spair::Context<Self>) {
        let (comp, element) = c.into_parts();
        element.nodes().match_if(|arm| match self.branch.as_ref() {
            Some(branch) => arm
                .render_on_arm_index(0)
                .render(branch)
                .button(|b| {
                    b.static_attributes()
                        .on_click(comp.handler(State::reset))
                        .static_nodes()
                        .render("Reset");
                })
                .done(),
            None => arm
                .render_on_arm_index(1)
                .button(|b| {
                    b.static_attributes()
                        .on_click(comp.handler(State::start_fetching))
                        .static_nodes()
                        .render("Click to fetch wasm-bindgen latest commit info");
                })
                .done(),
        });
    }
}

impl spair::Render<State> for &Branch {
    fn render(self, nodes: spair::Nodes<State>) -> spair::Nodes<State> {
        nodes
            .p(|p| {
                p.nodes()
                    .render("The latest commit to the wasm-bindgen ")
                    .render(&self.name)
                    .render(" branch is:");
            })
            .render(&self.commit)
    }
}

impl spair::Render<State> for &Commit {
    fn render(self, nodes: spair::Nodes<State>) -> spair::Nodes<State> {
        nodes.p(|p| {
            p.nodes()
                .render(&self.sha)
                .render(", authored by ")
                .render(&self.commit.author.name)
                .render(" (")
                .render(&self.commit.author.email)
                .render(")");
        })
    }
}

#[wasm_bindgen(start)]
pub fn start_fetch_example() {
    wasm_logger::init(wasm_logger::Config::default());
    let state = State { branch: None };
    spair::application::start(state, "root");
}
