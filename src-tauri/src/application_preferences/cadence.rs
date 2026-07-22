use std::time::Duration;
use tokio::sync::watch;

pub async fn wait_for_next_tick(receiver: &mut watch::Receiver<u64>) {
    loop {
        let interval_ms = *receiver.borrow_and_update();
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(interval_ms)) => return,
            changed = receiver.changed() => {
                if changed.is_err() {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn cadence_change_controls_the_next_tick() {
        let (sender, mut receiver) = watch::channel(1_500);
        let waiter = tokio::spawn(async move {
            wait_for_next_tick(&mut receiver).await;
            tokio::time::Instant::now()
        });

        tokio::time::advance(Duration::from_millis(500)).await;
        sender.send(1_000).unwrap();
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_millis(999)).await;
        assert!(!waiter.is_finished());
        tokio::time::advance(Duration::from_millis(1)).await;
        assert!(waiter.await.is_ok());
    }
}
