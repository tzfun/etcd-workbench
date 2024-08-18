#![cfg(test)]
mod test_connect {
    use etcd_client::Error;
    use crate::error::LogicError;
    use crate::etcd::etcd_connector::EtcdConnector;
    use crate::transport::connection::Connection;

    async fn get_connector() -> Result<EtcdConnector, LogicError> {
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
    async fn connect() -> Result<(), LogicError> {
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
    async fn get_history_version() -> Result<(), LogicError> {
        let connector = get_connector().await?;
        let history = connector.kv_get_history_versions(String::from("/asda/sdas/dasd/asd"), 8, 41).await?;
        println!("{:?}", history);
        Ok(())
    }

    #[tokio::test]
    async fn get_user() -> Result<(), LogicError> {
        let connector = get_connector().await?;
        let user = connector.user_list().await?;
        println!("{:?}", user);
        Ok(())
    }

    #[tokio::test]
    async fn get_cluster_info() -> Result<(), LogicError> {
        let connector = get_connector().await?;
        let cluster = connector.cluster_get().await?;
        println!("{:?}", cluster);
        Ok(())
    }

    #[tokio::test]
    async fn get_kv_paging() -> Result<(), LogicError> {
        let connector = get_connector().await?;
        let mut cursor = String::new();
        const LIMIT: i64 = 2;
        loop {
            let kv_list = connector.kv_get_all_keys_paging(cursor.clone(), LIMIT).await?;
            if let Some(kv) = kv_list.last() {
                cursor.clear();
                cursor.push_str(kv.key.clone().leak());
            }

            println!("{:?}", &kv_list);
            if kv_list.len() == 0 {
                break;
            }
            cursor.push('\0');
        }

        println!("finished");

        Ok(())
    }
}