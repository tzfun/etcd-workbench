#![cfg(test)]
mod test_connect {
    use etcd_client::Error;

    use crate::etcd::etcd_connector::EtcdConnector;
    use crate::transport::connection::Connection;

    async fn get_connector() -> Result<EtcdConnector, Error> {
        let connection: Connection = Connection {
            host: String::from("127.0.0.1"),
            port: 2379,
            namespace: None,
            user: None,
            tls: None,
            ssh: None,
        };
        EtcdConnector::new(connection).await
    }

    #[tokio::test]
    async fn connect() -> Result<(), Error> {
        let connector = get_connector().await?;
        let kvs = connector.kv_get_all_keys().await?;
        let mut first = false;
        for kv in kvs {
            let k = kv.key.clone();
            if !first {
                let kv = connector.kv_get(k).await?;
                println!("==> {:?}", kv);
                first = true;
            }
            println!("{:?}", kv);
        }

        Ok(())
    }

    #[tokio::test]
    async fn get_history_version() -> Result<(), Error> {
        let connector = get_connector().await?;
        let history = connector.kv_get_history_versions(String::from("/asda/sdas/dasd/asd"), 8, 41).await?;
        println!("{:?}", history);
        Ok(())
    }

    #[tokio::test]
    async fn get_user() -> Result<(), Error> {
        let connector = get_connector().await?;
        let user = connector.user_list().await?;
        println!("{:?}", user);
        Ok(())
    }
}