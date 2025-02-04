...
    // At the very end of the build_graph function, replace the return
    // Original code at line ~469:
    //     Ok(graph)
    // With the following to take ownership of the graph:
    Ok(std::mem::take(&mut graph))
...