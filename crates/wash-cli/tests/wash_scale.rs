mod common;

use std::vec;

use async_nats::jetstream::new;
use common::{TestWashInstance, HELLO_OCI_REF};

use anyhow::{Context, Result};
use serial_test::serial;
use tokio::process::Command;
use wash_lib::cli::output::{
    GetHostInventoriesCommandOutput, GetHostsCommandOutput, ScaleCommandOutput,
};

#[tokio::test]
#[serial]
#[cfg_attr(not(can_reach_ghcr_io), ignore = "ghcr.io is not reachable")]
async fn integration_scale_component_with_host_id_serial() -> Result<()> {
    let wash_instance = TestWashInstance::create().await?;

    let output = Command::new(env!("CARGO_BIN_EXE_wash"))
        .args([
            "scale",
            "component",
            HELLO_OCI_REF,
            "hello_component_id",
            wash_instance.host_id.as_str(),
            "--max",
            "10",
            "--output",
            "json",
            "--timeout-ms",
            "40000",
            "--skip-wait",
            "--ctl-port",
            &wash_instance.nats_port.to_string(),
        ])
        .kill_on_drop(true)
        .output()
        .await
        .context("failed to scale component")?;

    assert!(output.status.success(), "executed scale");

    let cmd_output: ScaleCommandOutput =
        serde_json::from_slice(&output.stdout).context("failed to parse output")?;
    assert!(cmd_output.success, "command returned accepted");

    // Give the host a couple of seconds to download the component bytes and start the component
    for retries in 0..5 {
        // get host inventory
        let output = Command::new(env!("CARGO_BIN_EXE_wash"))
            .args([
                "get",
                "inventory",
                "--output",
                "json",
                "--timeout-ms",
                "2000",
                "--ctl-port",
                &wash_instance.nats_port.to_string(),
            ])
            .kill_on_drop(true)
            .output()
            .await
            .context("failed to get host inventory")?;
        assert!(output.status.success(), "checked host inventory");

        let cmd_output: GetHostInventoriesCommandOutput =
            serde_json::from_slice(&output.stdout).context("failed to parse output")?;

        let components = cmd_output
            .inventories
            .into_iter()
            .next()
            .map(|i| i.components().clone())
            .unwrap_or_default();
        if components.is_empty() && retries > 4 {
            panic!("Should have started the component")
        } else if retries <= 4 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            continue;
        } else {
            assert_eq!(components.len(), 1);
            assert_eq!(components[0].max_instances(), 10);
            break;
        }
    }

    Ok(())
}

#[tokio::test]
#[serial]
#[cfg_attr(not(can_reach_ghcr_io), ignore = "ghcr.io is not reachable")]
async fn integration_scale_component_without_host_id_with_one_host_serial() -> Result<()> {
    let wash_instance = TestWashInstance::create().await?;

    let output = Command::new(env!("CARGO_BIN_EXE_wash"))
        .args([
            "scale",
            "component",
            HELLO_OCI_REF,
            "hello_component_id",
            "--max",
            "10",
            "--output",
            "json",
            "--timeout-ms",
            "40000",
            "--skip-wait",
            "--ctl-port",
            &wash_instance.nats_port.to_string(),
        ])
        .kill_on_drop(true)
        .output()
        .await
        .context("failed to scale component")?;

    assert!(output.status.success(), "executed scale");

    let cmd_output: ScaleCommandOutput =
        serde_json::from_slice(&output.stdout).context("failed to parse output")?;
    assert!(cmd_output.success, "command returned accepted");

    // Give the host a couple of seconds to download the component bytes and start the component
    for retries in 0..5 {
        // get host inventory
        let output = Command::new(env!("CARGO_BIN_EXE_wash"))
            .args([
                "get",
                "inventory",
                "--output",
                "json",
                "--timeout-ms",
                "2000",
                "--ctl-port",
                &wash_instance.nats_port.to_string(),
            ])
            .kill_on_drop(true)
            .output()
            .await
            .context("failed to get host inventory")?;
        assert!(output.status.success(), "checked host inventory");

        let cmd_output: GetHostInventoriesCommandOutput =
            serde_json::from_slice(&output.stdout).context("failed to parse output")?;

        let components = cmd_output
            .inventories
            .into_iter()
            .next()
            .map(|i| i.components().clone())
            .unwrap_or_default();
        if components.is_empty() && retries > 4 {
            panic!("Should have started the component")
        } else if retries <= 4 {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            continue;
        } else {
            assert_eq!(components.len(), 1);
            assert_eq!(components[0].max_instances(), 10);
            break;
        }
    }

    Ok(())
}

#[tokio::test]
#[serial]
#[cfg_attr(not(can_reach_ghcr_io), ignore = "ghcr.io is not reachable")]
async fn integration_scale_component_without_host_id_with_multiple_host_serial() -> Result<()> {
    let wash_instance = TestWashInstance::create().await?;
    eprintln!("wash_instance: {:?}", wash_instance);

    //wait 3 seconds for the first host to be registered
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    let _wash_instance_other = TestWashInstance::create_with_extra_args(vec![
        "--nats-port",
        wash_instance.nats_port.to_string().as_ref(),
        "--nats-connect-only",
        "--output",
        "json",
        "--detached",
        "--host-seed",
        wash_instance.host_seed.as_str(),
        "--cluster-seed",
        wash_instance.cluster_seed.as_str(),
        "--multi-local",
    ])
    .await?;
    // eprintln!("wash_instance_other: {:?}", _wash_instance_other);

    let output = Command::new(env!("CARGO_BIN_EXE_wash"))
        .args([
            "get",
            "hosts",
            "--output",
            "json",
            "--ctl-port",
            &wash_instance.nats_port.to_string(),
        ])
        .kill_on_drop(true)
        .output()
        .await
        .context("failed to execute get inventory")?;

    assert!(output.status.success(), "executed get claims query");

    let cmd_output: GetHostsCommandOutput = serde_json::from_slice(&output.stdout)?;
    assert!(cmd_output.success, "command returned success");
    assert!(cmd_output.hosts.len() == 2, "two host inventories returned");

    let output = Command::new(env!("CARGO_BIN_EXE_wash"))
        .args([
            "scale",
            "component",
            HELLO_OCI_REF,
            "hello_component_id",
            "--max",
            "10",
            "--output",
            "json",
            "--timeout-ms",
            "40000",
            "--skip-wait",
            "--ctl-port",
            &wash_instance.nats_port.to_string(),
        ])
        .kill_on_drop(true)
        .output()
        .await
        .context("failed to scale component")?;

    assert!(
        !output.status.success(),
        "unsuccessfully executed scale command"
    );

    Ok(())
}
