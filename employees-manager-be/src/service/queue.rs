use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicPublishArguments, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};

pub async fn send_message(queue_name: &str, content: String) -> Result<(), AppError> {
    // open a connection to RabbitMQ server
    let connection = Connection::open(&OpenConnectionArguments::default()).await?;
    connection
        .register_callback(DefaultConnectionCallback)
        .await?;
    // open a channel on the connection
    let channel = connection.open_channel(None).await.unwrap();
    channel.register_callback(DefaultChannelCallback).await?;
    // declare a queue
    let (queue_name, _, _) = channel
        .queue_declare(QueueDeclareArguments::new(queue_name))
        .await?
        .unwrap();
    let routing_key = &queue_name;
    let exchange_name = "amq.topic";
    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            exchange_name,
            routing_key,
        ))
        .await?;
    // publish message

    // create arguments for basic_publish
    let args = BasicPublishArguments::new(exchange_name, routing_key);

    channel
        .basic_publish(BasicProperties::default(), content.into_bytes(), args)
        .await?;
    connection.close().await
}
