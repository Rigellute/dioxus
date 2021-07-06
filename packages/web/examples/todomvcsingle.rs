//! Example: TODOVMC - One file
//! ---------------------------
//! This example shows how to build a one-file TODO MVC app with Dioxus and Recoil.
//! This project is confined to a single file to showcase the suggested patterns
//! for building a small but mighty UI with Dioxus without worrying about project structure.
//!
//! If you want an example on recommended project structure, check out the TodoMVC folder
//!
//! Here, we show to use Dioxus' Recoil state management solution to simplify app logic
#![allow(non_snake_case)]
use dioxus_core as dioxus;
use dioxus_web::dioxus::prelude::*;

use std::collections::HashMap;
use uuid::Uuid;

#[derive(PartialEq, Clone, Copy)]
pub enum FilterState {
    All,
    Active,
    Completed,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TodoItem {
    pub id: Uuid,
    pub checked: bool,
    pub contents: String,
}

// Declare our global app state
const TODO_LIST: AtomHashMap<Uuid, TodoItem> = |_| {};
const FILTER: Atom<FilterState> = |_| FilterState::All;
const TODOS_LEFT: Selector<usize> = |api| api.get(&TODO_LIST).len();

// Implement a simple abstraction over sets/gets of multiple atoms
struct TodoManager(RecoilApi);
impl TodoManager {
    fn add_todo(&self, contents: String) {
        let item = TodoItem {
            checked: false,
            contents,
            id: Uuid::new_v4(),
        };
        self.0.modify(&TODO_LIST, move |list| {
            list.insert(item.id, item);
        });
    }
    fn remove_todo(&self, id: &Uuid) {
        self.0.modify(&TODO_LIST, move |list| {
            list.remove(id);
        })
    }
    fn select_all_todos(&self) {
        self.0.modify(&TODO_LIST, move |list| {
            for item in list.values_mut() {
                item.checked = true;
            }
        })
    }
    fn toggle_todo(&self, id: &Uuid) {
        self.0.modify(&TODO_LIST, move |list| {
            list.get_mut(id).map(|item| item.checked = !item.checked)
        });
    }
    fn clear_completed(&self) {
        self.0.modify(&TODO_LIST, move |list| {
            *list = list.drain().filter(|(_, item)| !item.checked).collect();
        })
    }
    fn set_filter(&self, filter: &FilterState) {
        self.0.modify(&FILTER, move |f| *f = *filter);
    }
}

pub fn TodoList(cx: Context<()>) -> VNode {
    let draft = use_state(&cx, || "".to_string());
    let todos = use_read(&cx, &TODO_LIST);
    let filter = use_read(&cx, &FILTER);

    let todolist = todos
        .values()
        .filter(|item| match filter {
            FilterState::All => true,
            FilterState::Active => !item.checked,
            FilterState::Completed => item.checked,
        })
        .map(|item| {
            rsx!(TodoEntry {
                key: "{order}",
                id: item.id,
            })
        });

    rsx! { in cx,
        div {
            header {
                class: "header"
                h1 {"todos"}
                input {
                    class: "new-todo"
                    placeholder: "What needs to be done?"
                    value: "{draft}"
                    oninput: move |evt| draft.set(evt.value)
                }
            }
            {todolist}

            // rsx! accepts optionals, so we suggest the `then` method in place of ternary
            {(!todos.is_empty()).then(|| rsx!( FilterToggles {}) )}
        }
    }
}

#[derive(PartialEq, Props)]
pub struct TodoEntryProps {
    id: Uuid,
}

pub fn TodoEntry(cx: Context, props: &TodoEntryProps) -> VNode {
    let (is_editing, set_is_editing) = use_state_classic(&cx, || false);
    let todo = use_read(&cx, &TODO_LIST).get(&cx.id).unwrap();

    cx.render(rsx! (
        li {
            "{todo.id}"
            input {
                class: "toggle"
                type: "checkbox"
                "{todo.checked}"
            }
           {is_editing.then(|| rsx!(
                input {
                    value: "{todo.contents}"
                }
           ))}
        }
    ))
}

pub fn FilterToggles(cx: Context<()>) -> VNode {
    let reducer = TodoManager(use_recoil_api(cx));
    let items_left = use_read(cx, &TODOS_LEFT);

    let item_text = match items_left {
        1 => "item",
        _ => "items",
    };

    let toggles = rsx! {
        ul {
            class: "filters"
            li { class: "All", a { href: "", onclick: move |_| reducer.set_filter(&FilterState::All), "All" }}
            li { class: "Active", a { href: "active", onclick: move |_| reducer.set_filter(&FilterState::Active), "Active" }}
            li { class: "Completed", a { href: "completed", onclick: move |_| reducer.set_filter(&FilterState::Completed), "Completed" }}
        }
    };

    rsx! { in cx,
        footer {
            span {
                strong {"{items_left}"}
                span { "{item_text} left" }
            }
            {toggles}
        }
    }
}

pub fn Footer(cx: Context<()>) -> VNode {
    rsx! { in cx,
        footer { class: "info"
            p {"Double-click to edit a todo"}
            p {
                "Created by "
                a { "jkelleyrtp", href: "http://github.com/jkelleyrtp/" }
            }
            p {
                "Part of "
                a { "TodoMVC", href: "http://todomvc.com" }
            }
        }
    }
}

const APP_STYLE: &'static str = include_str!("./todomvc/style.css");

fn App(cx: Context<()>) -> VNode {
    use_init_recoil_root(cx, |_| {});
    rsx! { in cx,
        div { id: "app"
            TodoList {}
            Footer {}
        }
    }
}

fn main() {
    wasm_bindgen_futures::spawn_local(dioxus_web::WebsysRenderer::start(App));
}