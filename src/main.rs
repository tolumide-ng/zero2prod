use zero2prod::telemetry::init_subscriber;
use zero2prod::{startup::build::build, telemetry::get_subscriber};
use zero2prod::configuration::settings::get_configuration;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    
    let configuration = get_configuration().expect("Failed to read configurations");
    let server = build(configuration).await?;
    server.await?;

    Ok(())
}

