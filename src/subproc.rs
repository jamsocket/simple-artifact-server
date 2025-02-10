use anyhow::Result;
use nix::{sys::signal::Signal, unistd::Pid};
use std::{
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Weak,
    },
};
use tokio::{
    process::{Child, Command},
    select,
    sync::mpsc,
    task::JoinHandle,
};

#[derive(Debug, Clone)]
pub struct WrappedCommand {
    pub command: String,
    pub args: Vec<String>,
}

impl FromStr for WrappedCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = shlex::split(s).ok_or_else(|| anyhow::anyhow!("Invalid command"))?;
        let mut it = parts.into_iter();
        let command = it
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid command"))?;
        let args = it.collect::<Vec<_>>();
        Ok(Self {
            command: command.to_string(),
            args,
        })
    }
}

impl WrappedCommand {
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.command);
        command.args(&self.args);
        command
    }
}

pub enum WrappedServerCommand {
    /// Restart the subprocess (SIGKILL and spawn a new one)
    Restart,
    /// Interrupt the subprocess (SIGHUP)
    Interrupt,
    /// The state has changed, e.g. because a file was uploaded. This does not interrupt the
    /// process if it was running, but will cause it to try again if it has failed.
    StateChange,
}

pub struct WrappedServer {
    command: WrappedCommand,
    _handle: JoinHandle<()>,
    command_sender: mpsc::Sender<WrappedServerCommand>,
    running: AtomicBool,
    port: u16,
}

impl WrappedServer {
    pub fn running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub async fn interrupt(&self) {
        self.command_sender
            .send(WrappedServerCommand::Interrupt)
            .await
            .unwrap();
    }

    pub async fn restart(&self) {
        self.command_sender
            .send(WrappedServerCommand::Restart)
            .await
            .unwrap();
    }

    pub async fn state_change(&self) {
        self.command_sender
            .send(WrappedServerCommand::StateChange)
            .await
            .unwrap();
    }

    pub fn new(command: WrappedCommand, port: u16) -> Arc<Self> {
        let (command_sender, command_receiver) = mpsc::channel(32);

        Arc::new_cyclic(|server: &Weak<Self>| {
            let server = server.clone();
            WrappedServer {
                command: command,
                _handle: tokio::spawn(async move {
                    if let Err(e) = server.upgrade().unwrap().run_loop(command_receiver).await {
                        tracing::error!("Error running subprocess: {:?}", e);
                    }
                }),
                command_sender: command_sender,
                running: AtomicBool::new(false),
                port,
            }
        })
    }

    async fn handle_command(&self, child: &mut Child, command: WrappedServerCommand) -> Result<()> {
        match command {
            WrappedServerCommand::Restart => {
                tracing::info!("Restarting subprocess");
                child.kill().await?;
            }
            WrappedServerCommand::Interrupt => {
                tracing::info!("Interrupting subprocess");
                if let Some(id) = child.id() {
                    nix::sys::signal::kill(Pid::from_raw(id as i32), Signal::SIGHUP)?;
                }
            }
            WrappedServerCommand::StateChange => {}
        }
        Ok(())
    }

    async fn run_loop(
        self: Arc<Self>,
        mut command_receiver: mpsc::Receiver<WrappedServerCommand>,
    ) -> Result<()> {
        let mut child;
        loop {
            // Start process
            let mut command = self.command.command();
            command.env("PORT", self.port.to_string());
            // Prevent child from inheriting parent's signal handlers
            command.kill_on_drop(true);
            command.process_group(0); // Create new process group on Unix
            child = Some(command.spawn()?);
            self.running.store(true, Ordering::SeqCst);

            // Handle messages

            loop {
                if let Some(mut_child) = &mut child {
                    select! {
                        msg = command_receiver.recv() => {
                            if let Some(msg) = msg {
                                self.handle_command(mut_child, msg).await?;
                            } else {
                                return Ok(());
                            }
                        }
                        exit_code = mut_child.wait() => {
                            tracing::info!("Subprocess exited with code: {}. Attemping to restart.", exit_code?);
                            self.running.store(false, Ordering::SeqCst);
                            child = None;
                            continue;
                        }
                    }
                } else {
                    tracing::info!("Subprocess exited. Waiting for a signal before restarting.");
                    let msg = command_receiver.recv().await;
                    if msg.is_none() {
                        return Ok(());
                    }
                    break;
                }
            }
        }
    }
}
