use std::thread;
use std::time::{Duration, SystemTime};
use aws_sdk_elasticache::Client as ECacheClient;
use aws_sdk_elasticache::Error as ECacheError;

#[cfg(test)]
use aws_sdk_ec2::Client as Ec2Client;

#[cfg(test)]
use aws_sdk_ec2::Error as Ec2Error;

#[cfg(test)]
use aws_sdk_ec2::model::{InstanceType, InstanceStateName};

#[tokio::main]
async fn main() -> Result<(), ECacheError>{
    // XXX Not reproduced in main
    tracing_subscriber::fmt::init();

    let shared_config = aws_config::load_from_env().await;
    let client = ECacheClient::new(&shared_config);

    let now = SystemTime::now();
    let sec = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let name = format!("test-{}",sec);

    let req = client.create_cache_cluster().
              cache_cluster_id(&name).
              cache_node_type("cache.t3.micro").
              engine("memcached").
              engine_version("1.6.6").
              num_cache_nodes(2);
    let _resp = req.send().await?;

    let mut count = 0;
    loop {
        count += 1;
        println!("# of calls: {}",count);
        let req = client.describe_cache_clusters().
                  cache_cluster_id(&name);
        let resp = req.send().await?;
        match resp.cache_clusters {
            Some(cluster) => {
                if cluster[0].cache_cluster_status.as_ref().unwrap().contains("available") { }
            },
            None => { }
        }
        thread::sleep(Duration::from_millis(10000));

        if count == 100 { break }
    }

    let req = client.delete_cache_cluster().
              cache_cluster_id(&name);
    let _resp = req.send().await?;

    Ok(())
}

#[tokio::test]
async fn test_client_elasticache() -> Result<(), ECacheError>{
    tracing_subscriber::fmt::init();

    let shared_config = aws_config::load_from_env().await;
    let client = ECacheClient::new(&shared_config);

    let now = SystemTime::now();
    let sec = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
    let name = format!("test-{}",sec);

    // XXX Skip this if you alreaday have a cluster
    let req = client.create_cache_cluster().
              cache_cluster_id(&name).
              cache_node_type("cache.t3.micro").
              engine("memcached").
              engine_version("1.6.6").
              num_cache_nodes(2);
    let _resp = req.send().await?;

    let mut count = 0;
    loop {
        count += 1;
        println!("# of calls: {}",count);

        // XXX Not reproduced with this
        //let client = ECacheClient::new(&shared_config);

        let req = client.describe_cache_clusters().
                  cache_cluster_id(&name);
        let resp = req.send().await?;
        match resp.cache_clusters {
            Some(cluster) => {
                //if cluster[0].cache_cluster_status.as_ref().unwrap().contains("available") { break }
                if cluster[0].cache_cluster_status.as_ref().unwrap().contains("available") { }
            },
            None => { }
        }

        thread::sleep(Duration::from_millis(10000));

        // XXX Not reproduced with this instead of above
        //thread::sleep(Duration::from_millis(5000));

        if count == 100 { break }
    }

    let req = client.delete_cache_cluster().
              cache_cluster_id(&name);
    let _resp = req.send().await?;

    Ok(())
}

#[tokio::test]
async fn test_client_ec2() -> Result<(),Ec2Error>{
    // XXX Not reproduced in this test with EC2 API
    tracing_subscriber::fmt::init();

    let shared_config = aws_config::load_from_env().await;
    let client = Ec2Client::new(&shared_config);

    let req = client.run_instances().
              image_id("ami-03d79d440297083e3").
              instance_type(InstanceType::from("t3.micro")).
              min_count(1).
              max_count(1);
    let resp = req.send().await?;

    thread::sleep(Duration::from_millis(1000));

    let id = match resp.instances {
        Some(instance) => { instance[0].instance_id.as_ref().unwrap().clone() },
        None => { String::from("") }
    };


    let mut count = 0;
    loop {
        count += 1;
        println!("# of calls : {}",count);
        let req = client.describe_instances().
                  instance_ids(&id);
        let resp = req.send().await?;
        match resp.reservations().unwrap()[0].instances() {
            Some(instance) => {
                match instance[0].state.as_ref().unwrap().name().unwrap() {
                    //InstanceStateName::Running => { break; },
                    InstanceStateName::Running => { },
                    _ => { }
                }
            },
            None => { }
        }
        thread::sleep(Duration::from_millis(10000));

        if count == 100 { break }
    }

    let req = client.terminate_instances().
              instance_ids(&id);
    let _resp = req.send().await?;

    Ok(())
}

