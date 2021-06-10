/// Manages a cursor over a list of contacts.
use crate::generated::common::ContactInfo;
use log::{debug, error};
use rusqlite::{Connection, Rows, Statement};
use std::sync::mpsc::{channel, Sender};
use threadpool::ThreadPool;

enum CursorCommand {
    Next(Sender<Option<Vec<ContactInfo>>>),
    Stop,
}

pub struct ContactDbCursor {
    sender: Sender<CursorCommand>,
}

impl Iterator for ContactDbCursor {
    type Item = Vec<ContactInfo>;

    fn next(&mut self) -> Option<Self::Item> {
        let (sender, receiver) = channel();

        let _ = self.sender.send(CursorCommand::Next(sender));
        match receiver.recv() {
            Ok(msg) => msg,
            Err(err) => {
                // Since the cursor releases resources eagerly once it reaches the end of
                // the contact list, this can fail without being an issue.
                debug!("Cursor is already closed: {}", err);
                None
            }
        }
    }
}

impl Drop for ContactDbCursor {
    fn drop(&mut self) {
        debug!("ContactDbCursor::drop");
        let _ = self.sender.send(CursorCommand::Stop);
    }
}

enum ProcessCommandResult {
    ForceExit,
    Break,
}

impl ContactDbCursor {
    pub fn new<F: 'static>(
        batch_size: i64,
        only_main_data: bool,
        pool: &ThreadPool,
        prepare: F,
    ) -> Self
    where
        F: Fn(&Connection) -> Option<Statement> + Send,
    {
        let (sender, receiver) = channel();
        pool.execute(move || {
            let mut db = crate::db::create_db();
            let connection = db.mut_connection();
            let mut statement = match prepare(&connection) {
                Some(statement) => statement,
                None => {
                    // Return an empty cursor.
                    // We use a trick here where we create a request that will always return 0 results.
                    connection
                        .prepare("SELECT contact_id FROM contact_main where contact_id = ''")
                        .unwrap()
                }
            };
            let mut rows = statement.raw_query();
            let mut force_exit = false;
            loop {
                match receiver.recv() {
                    Ok(cmd) => {
                        match Self::process_command(
                            &cmd,
                            connection,
                            &mut rows,
                            only_main_data,
                            batch_size,
                        ) {
                            Some(ProcessCommandResult::Break) => break,
                            Some(ProcessCommandResult::ForceExit) => {
                                force_exit = true;
                            }
                            None => {}
                        }
                    }
                    Err(err) => {
                        error!("receiver.recv error: {}", err);
                        break;
                    }
                }

                if force_exit {
                    break;
                }
            }
            debug!("Exiting contacts cursor thread");
        });
        Self { sender }
    }

    fn process_command(
        cmd: &CursorCommand,
        connection: &Connection,
        rows: &mut Rows,
        only_main_data: bool,
        batch_size: i64,
    ) -> Option<ProcessCommandResult> {
        match cmd {
            CursorCommand::Stop => Some(ProcessCommandResult::Break),
            CursorCommand::Next(sender) => {
                let mut results = vec![];
                loop {
                    match rows.next() {
                        Ok(None) => {
                            // We are out of items. Send the current item list or reject directly.
                            if results.is_empty() {
                                debug!("ContactDbCursor, empty result, reject directly");
                                let _ = sender.send(None);
                            } else {
                                debug!("Send results with len:{}", results.len());
                                let _ = sender.send(Some(results));
                            }
                            // Force leaving the outer loop since we can release resources at this
                            // point.
                            return Some(ProcessCommandResult::ForceExit);
                        }
                        Ok(Some(row)) => {
                            if let Ok(id) = crate::db::row_to_contact_id(row) {
                                debug!("current id is {}", id);
                                let mut contact = ContactInfo::default();
                                if let Err(err) = contact.fill_main_data(&id, connection) {
                                    error!(
                                        "ContactDbCursor fill_main_data error: {}, continue",
                                        err
                                    );
                                    continue;
                                }

                                if !only_main_data {
                                    if let Err(err) = contact.fill_additional_data(&id, connection)
                                    {
                                        error!("ContactDbCursor fill_additional_data error: {}, continue", err);
                                        continue;
                                    }
                                }
                                results.push(contact);

                                if results.len() == batch_size as usize {
                                    let _ = sender.send(Some(results));
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            error!("Failed to fetch row: {}", err);
                            let _ = sender.send(None);
                        }
                    }
                }
                None
            }
        }
    }
}
