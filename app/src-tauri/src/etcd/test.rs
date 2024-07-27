#![cfg(test)]
mod test_connect {
    use etcd_client::Error;
    use crate::etcd::etcd_connector::EtcdConnector;
    use crate::transport::connection::Connection;

    #[tokio::test]
    async fn connect() -> Result<(), Error> {
        let connection: Connection = Connection {
            host: String::from("127.0.0.1"),
            port: 2379,
            namespace: None,
            user: None,
            tls: None,
            ssh: None
        };
        let connector = EtcdConnector::new(connection).await?;
        let kvs = connector.get_all_keys().await?;
        println!("{:?}", kvs);
        Ok(())
    }
}