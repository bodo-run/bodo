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
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let result: Result<(), BodoError> = async {
        let args = Args::parse();

        // If auto_watch is set, turn on watch mode
        let watch_mode = if args.auto_watch { true } else { args.watch };

        let config = BodoConfig {
            root_script: None,
            scripts_dirs: Some(vec!["scripts/".into()]),
            tasks: HashMap::new(),
        };

        let mut graph_manager = GraphManager::new();
        graph_manager.build_graph(config).await?;

        if args.list {
            graph_manager.register_plugin(Box::new(PrintListPlugin));
            graph_manager.run_plugins(None).await?;
            return Ok(());
        }

        // register plugins in this specific order:
        graph_manager.register_plugin(Box::new(EnvPlugin::new()));
        graph_manager.register_plugin(Box::new(PathPlugin::new()));
        graph_manager.register_plugin(Box::new(ConcurrentPlugin::new()));
        graph_manager.register_plugin(Box::new(PrefixPlugin::new()));
        graph_manager.register_plugin(Box::new(WatchPlugin::new()));
        graph_manager.register_plugin(Box::new(ExecutionPlugin::new()));
        graph_manager.register_plugin(Box::new(TimeoutPlugin::new()));

        // Run the task
        let task_name = get_task_name(&args, &graph_manager)?;
        let mut options = serde_json::Map::new();
        options.insert("task".into(), serde_json::Value::String(task_name.clone()));

        // The PluginConfig now honors our local watch_mode
        let plugin_config = PluginConfig {
            fail_fast: true,
            watch: watch_mode,
            list: false,
            options: Some(options),
        };

        graph_manager.run_plugins(Some(plugin_config)).await?;

        Ok(())
    }
    .await;

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
