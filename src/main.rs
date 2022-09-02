use async_std::prelude::FutureExt;
use zero2prod::configuration::get_configuration;
use zero2prod::issue_delivery_worker::run_worker_until_stopped;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[async_std::main]
async fn main() -> tide::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application =
        async_std::task::spawn(Application::build(configuration.clone())?.run_until_stopped());
    let worker = async_std::task::spawn(run_worker_until_stopped(configuration));

    let one_task = application.race(worker);
    one_task.await?;

    Ok(())
}
