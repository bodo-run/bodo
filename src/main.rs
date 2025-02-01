use bodo::{
    cli::{get_task_name, Args},
    config::BodoConfig,
    manager::GraphManager,
    plugin::PluginConfig,
    plugins::{
        concurrent_plugin::ConcurrentPlugin, env_plugin::EnvPlugin,
        execution_plugin::ExecutionPlugin, path_plugin::PathPlugin, prefix_plugin::PrefixPlugin,
        print_list_plugin::PrintListPlugin, timeout_plugin::TimeoutPlugin,
        watch_plugin::WatchPlugin,
    },
    BodoError,
};
use clap::Parser;
use log::{error, LevelFilter};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize env_logger with a default "info" level;
    // if --debug is used, set "debug" level for our crate.
    if args.debug {
        std::env::set_var("RUST_LOG", "bodo=debug");
    } else {
        // If the user hasn't set RUST_LOG externally, default to "info"
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", "bodo=info");
        }
    }
    env_logger::Builder::from_default_env()
        .filter_module(
            "bodo",
            if args.debug {
                LevelFilter::Debug
            } else {
                LevelFilter::Info
            },
        )
        .init();

    let result: Result<(), BodoError> = async {
        let watch_mode = if args.auto_watch { true } else { args.watch };

        let config = BodoConfig {
            root_script: None,
            scripts_dirs: Some(vec!["scripts/".into()]),
            tasks: HashMap::new(),
        };

        let graph_manager = Arc::new(Mutex::new(GraphManager::new()));
        graph_manager.lock().await.build_graph(config).await?;

        if args.list {
            graph_manager
                .lock()
                .await
                .register_plugin(Box::new(PrintListPlugin));
            graph_manager.lock().await.run_plugins(None).await?;
            return Ok(());
        }

        let mut manager = graph_manager.lock().await;
        manager.register_plugin(Box::new(EnvPlugin::new()));
        manager.register_plugin(Box::new(PathPlugin::new()));
        manager.register_plugin(Box::new(ConcurrentPlugin::new()));
        manager.register_plugin(Box::new(PrefixPlugin::new()));
        manager.register_plugin(Box::new(WatchPlugin::new(
            Arc::clone(&graph_manager),
            watch_mode,
            true,
        )));
        manager.register_plugin(Box::new(ExecutionPlugin::new()));
        manager.register_plugin(Box::new(TimeoutPlugin::new()));

        let task_name = get_task_name(&args, &manager)?;
        let mut options = serde_json::Map::new();
        options.insert("task".into(), serde_json::Value::String(task_name.clone()));

        let plugin_config = PluginConfig {
            fail_fast: true,
            watch: watch_mode,
            list: false,
            options: Some(options),
        };

        manager.run_plugins(Some(plugin_config)).await?;

        Ok(())
    }
    .await;

    if let Err(e) = result {
        error!("Error: {}", e);
        std::process::exit(1);
    }
}
