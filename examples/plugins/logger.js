module.exports = {
  onBeforeTaskRun: function(opts) {
    console.log(`[JS Logger] Starting task: ${opts.taskName} at ${opts.timestamp}`);
  },

  onAfterTaskRun: function(opts) {
    console.log(`[JS Logger] Task ${opts.taskName} completed with status: ${opts.status}`);
  },

  onError: function(opts) {
    console.error(`[JS Logger] Task ${opts.taskName} failed: ${opts.error}`);
  }
}; 