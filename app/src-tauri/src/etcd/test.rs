#![cfg(test)]
mod test_connect {
    use etcd_client::Error;
    use crate::error::LogicError;
    use crate::etcd::etcd_connector::EtcdConnector;
    use crate::etcd::etcd_connector_handler::EtcdConnectorHandler;
    use crate::transport::connection::Connection;

    async fn get_connector() -> Result<EtcdConnector, LogicError> {
        let connection: Connection = Connection {
            host: String::from("127.0.0.1"),
            port: 2379,
            namespace: Some(String::from("/tz_mac")),
            user: None,
            tls: None,
            ssh: None,
        };
        EtcdConnector::new(connection, EtcdConnectorHandler::default()).await
    }

    #[tokio::test]
    async fn connect() -> Result<(), LogicError> {
        let mut connector = get_connector().await?;
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
    async fn get_history_version() -> Result<(), LogicError> {
        let mut connector = get_connector().await?;
        let history = connector.kv_get_history_versions(String::from("/asda/sdas/dasd/asd"), 8, 41).await?;
        println!("{:?}", history);
        Ok(())
    }

    #[tokio::test]
    async fn get_user() -> Result<(), LogicError> {
        let mut connector = get_connector().await?;
        let user = connector.user_list().await?;
        println!("{:?}", user);
        Ok(())
    }

    #[tokio::test]
    async fn get_cluster_info() -> Result<(), LogicError> {
        let mut connector = get_connector().await?;
        let cluster = connector.cluster_get().await?;
        println!("{:?}", cluster);
        Ok(())
    }

    #[tokio::test]
    async fn get_kv_paging() -> Result<(), LogicError> {
        let mut connector = get_connector().await?;
        let mut cursor = String::new();
        let mut page = 1;
        const LIMIT: i64 = 2;
        loop {
            let kv_list = connector.kv_get_all_keys_paging(cursor.clone(), LIMIT).await?;
            if let Some(kv) = kv_list.last() {
                cursor.clear();
                cursor.push_str(kv.key.clone().leak());
            }

            println!("{} ==> {:?}", page, &kv_list);
            if kv_list.len() < LIMIT as usize {
                break;
            }
            page += 1;
        }

        println!("finished");

        Ok(())
    }

    #[tokio::test]
    async fn put_huge_kvs() -> Result<(), LogicError> {
        let mut connector = get_connector().await?;
        
        for i in 0..20000 {
            let mut key = String::new();
            if i % 3 == 0 {
                key.push_str("/deep1/");
            } else if i % 7 == 0 {
                key.push_str("/deep2/");
            } else if i % 2 == 0 {
                key.push_str("/deep3/");
            } else {
                key.push_str("/deep4/");
            }
            key.push_str(format!("key-{}.txt", i).as_str());
            
            let value = format!("value {}", i);
            connector.kv_put(key, value, None).await?;
        }
        println!("finished");
        Ok(())
    }
}