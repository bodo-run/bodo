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
use std::{collections::HashMap, process::exit};

fn main() {
    let args = Args::parse();

    if args.debug {
        std::env::set_var("RUST_LOG", "bodo=debug");
    } else if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "bodo=info");
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

    if let Err(e) = run(args) {
        error!("Error: {}", e);
        exit(1);
    }
}

fn run(args: Args) -> Result<(), BodoError> {
    let watch_mode = if std::env::var("BODO_NO_WATCH").is_ok() {
        false
    } else if args.auto_watch {
        true
    } else {
        args.watch
    };

    let root_script = std::env::var("BODO_ROOT_SCRIPT")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "scripts/script.yaml".to_string());

    let scripts_dirs = std::env::var("BODO_SCRIPTS_DIRS")
        .map(|s| s.split(',').map(|s| s.to_string()).collect())
        .unwrap_or_else(|_| vec!["scripts/".to_string()]);

    // Read the root script file if it exists
    let default_task = if let Ok(content) = std::fs::read_to_string(&root_script) {
        if let Ok(config) = serde_yaml::from_str::<BodoConfig>(&content) {
            config.default_task
        } else {
            None
        }
    } else {
        None
    };

    let config = BodoConfig {
        root_script: Some(root_script),
        scripts_dirs: Some(scripts_dirs),
        default_task,
        tasks: HashMap::new(),
        env: HashMap::new(),
        exec_paths: vec![],
    };

    let mut graph_manager = GraphManager::new();
    graph_manager.build_graph(config)?;

    if args.list {
        graph_manager.register_plugin(Box::new(PrintListPlugin));
        graph_manager.run_plugins(None)?;
        return Ok(());
    }

    // Register all normal plugins
    graph_manager.register_plugin(Box::new(EnvPlugin::new()));
    graph_manager.register_plugin(Box::new(PathPlugin::new()));
    graph_manager.register_plugin(Box::new(ConcurrentPlugin::new()));
    graph_manager.register_plugin(Box::new(PrefixPlugin::new()));
    graph_manager.register_plugin(Box::new(WatchPlugin::new(watch_mode, true)));
    graph_manager.register_plugin(Box::new(ExecutionPlugin::new()));
    graph_manager.register_plugin(Box::new(TimeoutPlugin::new()));

    let task_name = get_task_name(&args, &graph_manager)?;

    // Apply any CLI arguments to the task before running plugins
    graph_manager.apply_task_arguments(&task_name, &args.args)?;

    let mut options = serde_json::Map::new();
    options.insert("task".into(), serde_json::Value::String(task_name.clone()));

    let plugin_config = PluginConfig {
        fail_fast: true,
        watch: watch_mode,
        list: false,
        options: Some(options),
    };

    if args.dry_run {
        // Run dry-run simulation
        let report = graph_manager.run_dry_run(Some(plugin_config))?;

        // Output the dry-run report
        if args.debug {
            println!("Dry-run Report (Debug):");
            println!("{:#?}", report);
        } else {
            println!("Dry-run Report:");
            for plugin_report in &report.reports {
                println!("Plugin: {}", plugin_report.plugin_name);
                println!("Simulated Actions:");
                for action in &plugin_report.simulated_actions {
                    println!("  - {}: {}", action.action_type, action.description);
                    for (key, value) in &action.details {
                        println!("    {}: {}", key, value);
                    }
                }
                if !plugin_report.dependencies.is_empty() {
                    println!("Dependencies: {:?}", plugin_report.dependencies);
                }
                if !plugin_report.warnings.is_empty() {
                    println!("Warnings:");
                    for warning in &plugin_report.warnings {
                        println!("  - {}", warning);
                    }
                }
                println!();
            }
        }
    } else {
        // Normal execution flow
        graph_manager.run_plugins(Some(plugin_config))?;
    }

    Ok(())
}
