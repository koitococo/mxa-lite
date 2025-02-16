use std::str::FromStr;

use crate::{
    executor::handle_event,
    messages::{AgentResponse, AgentResponsePayload, ControllerRequest},
};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::{debug, error, info, trace, warn};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

type WebSocketTx = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

#[derive(Debug, Clone)]
struct AsyncResponder {
    tx: std::sync::Arc<tokio::sync::Mutex<WebSocketTx>>,
}

impl AsyncResponder {
    fn new(tx: WebSocketTx) -> Self {
        AsyncResponder {
            tx: std::sync::Arc::new(tokio::sync::Mutex::new(tx)),
        }
    }

    async fn respond(self, msg: Message) -> Result<()> {
        let mut guard = self.tx.lock().await;
        guard.send(msg).await?;
        Ok(())
    }
}

pub(crate) struct Context {
    pub id: u64,
    pub request: Request,
    responder: AsyncResponder,
}

impl Context {
    pub(crate) async fn respond(&self, response: Response) -> Result<()> {
        let msg = match response {
            Response::Text(r) => Message::Text(r.to_string()),
        };
        self.responder.clone().respond(msg).await
    }

    pub(crate) async fn respond2(&self, ok: bool, payload: AgentResponsePayload) -> () {
        if let Err(e) = self
            .respond(Response::Text(AgentResponse {
                id: self.id,
                ok,
                payload,
            }))
            .await
        {
            warn!("Failed to respond request[id={}]: {}", self.id, e);
        }
    }
}

#[derive(Debug)]
pub(crate) enum Request {
    Text(ControllerRequest),
}

#[derive(Debug)]
pub(crate) enum Response {
    Text(AgentResponse),
}

async fn handle_msg(ws_msg: Message, responder: AsyncResponder) -> Result<bool> {
    debug!("Received message: {:?}", ws_msg);
    match ws_msg {
        Message::Text(msg) => {
            trace!("Received text message from controller");
            match ControllerRequest::from_str(msg.as_str()) {
                Ok(event_msg) => {
                    info!("Received event: {:?}", event_msg);
                    let ctx = Context {
                        id: event_msg.id,
                        request: Request::Text(event_msg),
                        responder,
                    };
                    tokio::spawn(async move {
                        if let Err(e) = handle_event(ctx).await {
                            error!("Failed to handle event: {}", e);
                        }
                    });
                }
                Err(err) => {
                    error!("Failed to parse message: {}", err);
                    if let Err(e) = responder
                        .respond(Message::Text(
                            AgentResponse {
                                id: u64::MAX,
                                ok: false,
                                payload: AgentResponsePayload::None,
                            }
                            .to_string(),
                        ))
                        .await
                    {
                        error!("Failed to respond to malformed message: {}", e);
                    }
                }
            }
        }
        Message::Binary(_) => {
            debug!("Received binary message from controller"); // TODO: Handle binary message
        }
        Message::Ping(msg) => {
            responder.respond(Message::Pong(msg)).await?;
            debug!("Received Ping from controller, Pong sent");
        }
        Message::Pong(_) => {
            debug!("Received Pong from controller");
        }
        Message::Close(_) => {
            // Unexpected close message from controller. Connection closing should be initiated by agent or by specific event from controller
            // TODO: handle unexpected close
            warn!("Websocket connection closed, retry");
        }
        Message::Frame(_) => {
            // maybe a malformed message, ignore
            warn!("Received a malformed message from controller, ignored",)
        }
    }
    Ok(true)
}

async fn handle_conn(
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) -> Result<()> {
    let (tx, mut rx) = ws.split();
    let responder = AsyncResponder::new(tx);
    trace!("Websocket connected to controller. Begin to handle message loop");
    while let Some(event) = rx.next().await {
        match event {
            Ok(ws_msg) => match handle_msg(ws_msg, responder.clone()).await {
                Ok(c) => {
                    if !c {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to handle message: {}", e);
                }
            },
            Err(err) => {
                error!("Failed to receive message: {}", err);
            }
        }
    }
    Ok(())
}

pub(crate) async fn agent_main(ws_url: String, host_id: String) -> Result<()> {
    info!("Use Controller URL: {}", ws_url);
    let ws_url = format!("{}/?host_id={}", ws_url, host_id);
    loop {
        let ws_url = ws_url.clone();
        info!("Connecting to controller websocket: {}", ws_url);
        for retry in 0..5 {
            match connect_async(ws_url.clone()).await {
                Ok((ws, _)) => {
                    handle_conn(ws).await?;
                    break;
                }
                Err(err) => {
                    error!("Failed to connect to controller: {}", err);
                    tokio::time::sleep(std::time::Duration::from_secs(
                        ((1.5f32).powi(retry) * 3f32 + 5f32) as u64,
                    ))
                    .await;
                }
            }
        }
    }
}
