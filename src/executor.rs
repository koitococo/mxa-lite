use anyhow::Result;
use log::warn;

use crate::messages::{AgentResponse, ControllerRequest};
use crate::net::{Context, Request, Response};
use crate::utils::{download_file, execute_shell, upload_file};

struct FileDownloadUploadTask {
    url: String,
    path: String,
}

impl FileDownloadUploadTask {
    async fn handle_download(self, ctx: Context) -> Result<()> {
        tokio::spawn(async move {
            if let Err(err) = download_file(&self.url, &self.path).await {
                warn!(
                    "Failed to download file from '{}' to '{}': {}",
                    self.url, self.path, err
                );
                if let Err(e) = ctx
                    .respond(Response::Text(AgentResponse {
                        id: ctx.id,
                        ok: false,
                        payload: crate::messages::AgentResponsePayload::FileOperationResponse(
                            crate::messages::FileOperationResponse { success: false },
                        ),
                    }))
                    .await
                {
                    warn!("Failed to respond to file download: {}", e);
                }
            }
            if let Err(e) = ctx
                .respond(Response::Text(AgentResponse {
                    id: ctx.id,
                    ok: true,
                    payload: crate::messages::AgentResponsePayload::FileOperationResponse(
                        crate::messages::FileOperationResponse { success: true },
                    ),
                }))
                .await
            {
                warn!("Failed to respond to file download: {}", e);
            }
        });
        Ok(())
    }

    async fn handle_upload(self, ctx: Context) -> Result<()> {
        tokio::spawn(async move {
            if let Err(err) = upload_file(&self.url, &self.path).await {
                warn!(
                    "Failed to upload file from '{}' to '{}': {}",
                    self.path, self.url, err
                );
                if let Err(e) = ctx
                    .respond(Response::Text(AgentResponse {
                        id: ctx.id,
                        ok: false,
                        payload: crate::messages::AgentResponsePayload::FileOperationResponse(
                            crate::messages::FileOperationResponse { success: false },
                        ),
                    }))
                    .await
                {
                    warn!("Failed to respond to file upload: {}", e);
                }
            }
            if let Err(e) = ctx
                .respond(Response::Text(AgentResponse {
                    id: ctx.id,
                    ok: true,
                    payload: crate::messages::AgentResponsePayload::FileOperationResponse(
                        crate::messages::FileOperationResponse { success: true },
                    ),
                }))
                .await
            {
                warn!("Failed to respond to file upload: {}", e);
            }
        });
        Ok(())
    }
}

struct ExecuteTask {
    cmd: String,
}

impl ExecuteTask {
    async fn handle(self, ctx: Context) -> Result<()> {
        tokio::spawn(async move {
            match execute_shell(&self.cmd).await {
                Ok(code) => {
                    if let Err(e) = ctx
                        .respond(Response::Text(AgentResponse {
                            id: ctx.id,
                            ok: true,
                            payload:
                                crate::messages::AgentResponsePayload::CommandExecutionResponse(
                                    crate::messages::CommandExecutionResponse {
                                        code,
                                        output: "".to_string(),
                                    },
                                ),
                        }))
                        .await
                    {
                        warn!("Failed to respond to command execution: {}", e);
                    }
                }
                Err(err) => {
                    warn!("Failed to execute command {}: {}", self.cmd, err);
                    if let Err(e) = ctx.respond(Response::Text(AgentResponse {
                        id: ctx.id,
                        ok: false,
                        payload: crate::messages::AgentResponsePayload::CommandExecutionResponse(
                            crate::messages::CommandExecutionResponse {
                                code: -1,
                                output: "".to_string(),
                            },
                        ),
                    }))
                    .await {
                        warn!("Failed to respond to command execution: {}", e);
                    }
                }
            }
        });
        Ok(())
    }
}
enum Task {
    Download(FileDownloadUploadTask),
    Upload(FileDownloadUploadTask),
    Execute(ExecuteTask),
}

impl Task {
    async fn handle(self, ctx: Context) -> Result<()> {
        match self {
            Task::Download(task) => task.handle_download(ctx).await,
            Task::Upload(task) => task.handle_upload(ctx).await,
            Task::Execute(task) => task.handle(ctx).await,
        }
    }
}

impl TryFrom<&ControllerRequest> for Task {
    type Error = ();

    fn try_from(msg: &ControllerRequest) -> Result<Self, Self::Error> {
        match &msg.payload {
            crate::messages::ControllerRequestPayload::FileOperationRequest(req) => {
                match req.operation {
                    crate::messages::FileOperation::Download => {
                        Ok(Task::Download(FileDownloadUploadTask {
                            url: req.url.clone(),
                            path: req.path.clone(),
                        }))
                    }
                    crate::messages::FileOperation::Upload => {
                        Ok(Task::Upload(FileDownloadUploadTask {
                            url: req.url.clone(),
                            path: req.path.clone(),
                        }))
                    }
                }
            }
            crate::messages::ControllerRequestPayload::CommandExecutionRequest(req) => {
                Ok(Task::Execute(ExecuteTask {
                    cmd: req.command.clone(),
                }))
            }
        }
    }
}

impl TryFrom<&Request> for Task {
    type Error = ();

    fn try_from(msg: &Request) -> Result<Self, Self::Error> {
        match msg {
            Request::Text(msg) => Task::try_from(msg),
        }
    }
}

pub(crate) async fn handle_event(ctx: Context) -> Result<()> {
    let task = Task::try_from(&ctx.request);
    match task {
        Ok(task) => task.handle(ctx).await,
        Err(_) => {
            warn!("Received an invalid task: {:?}", ctx.request);
            Ok(())
        }
    }
}
