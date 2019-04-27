use sharedlib::head_rpc::MESSAGES;
use std::{thread, time, io};

/*
 * This function is used to periodically end a round,
 * flush the messages to the next server in the chain,
 * and begin tracking messages for the next round.
 */
pub async fn round_status_check() -> io::Result<()> {
	thread::sleep(time::Duration::from_millis(1000));
	// sleep until round completes
	{
		// acquire lock on MESSAGES
		let arr = MESSAGES.lock().unwrap();
		// permute the messages *before* proceeding further
		// begin sending messages to the intermediate server in chunks
		// increment the round number
		// when we are done drop the lock on MESSAGES by ending the scope
	}

	Ok()
}