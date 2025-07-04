#![allow(non_snake_case)]
use dioxus::prelude::*;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
//use gloo_timers::future::TimeoutFuture;

fn main() {
    dioxus::launch(App);
}

#[component]
pub fn App() -> Element {
    rsx!(
        document::Stylesheet { href: asset!("/assets/main.css") }

        Home {}
    )
}

#[component]
fn Home() -> Element {
    let mut message_list = use_signal(std::vec::Vec::new);
    let mut message_content = use_signal(String::new);
    let mut receiver_ws = use_signal(|| None);

    let mut name = use_signal(String::new);
    let mut has_name = use_signal(|| false);

    let chat_client = use_coroutine(move |mut rx: UnboundedReceiver<String>| async move {
        let (mut sender, receiver) = WebSocket::open("ws://localhost:3000/chat").unwrap().split();
        //TimeoutFuture::new(500).await;
        receiver_ws.set(Some(receiver));
        while let Some(msg) = rx.next().await {
            let message = format!("{}:{}", name, msg);
            sender.send(Message::Text(message)).await.unwrap();
        }
    });

    use_effect(move || {
        //read subscribes to reactivity making the use effect rerender when ws changes to Some, jsut take would ran only once for possible None, whcich can happen in race condition.
        if receiver_ws.read().is_some() {
            if let Some(mut receiver) = receiver_ws.take() {
                spawn(async move {
                    while let Some(Ok(Message::Text(content))) = receiver.next().await {
                        message_list.write().push(content);
                    }
                });
            }
        }
    });
    rsx!(
        if !has_name() {
            div { class: "chat-container",
                div { class: "chat input-name",
                    input {
                        r#type: "text",
                        value: name,
                        placeholder: "Enter Your Name ...",
                        oninput: move |e| name.set(e.value()),
                    }
                    button {
                        onclick: move |_| has_name.set(true),
                        disabled: if name().trim() == "" { true },
                        "Join Chat"
                    }
                }
            }
        } else {
            div { class: "chat-container",
                div { class: "chat",
                    div { class: "message-container",
                        {
                            message_list()
                                .iter()
                                .rev()
                                .map(|item| {
                                    let username = item.split(":").collect::<Vec<&str>>()[0];
                                    rsx! {
                                        p { class: "message-item", class: if username == name() { "user-message" }, "{item}" }
                                    }
                                })
                        }
                    }
                    div { class: "input-container",
                        input {
                            r#type: "text",
                            value: message_content,
                            placeholder: name,
                            oninput: move |e| message_content.set(e.value()),
                        }
                        button {
                            onclick: move |_| {
                                chat_client.send(message_content());
                                message_content.set(String::new());
                            },
                            disabled: if message_content().trim() == "" { true },
                            "Send"
                        }
                    }
                }
            }
        }
    )
}
