module BodoPlugin
  class << self
    def onBeforeTaskRun(opts)
      notify("Starting task: #{opts['taskName']}")
    end

    def onAfterTaskRun(opts)
      status = opts['status'] == 0 ? 'successfully' : 'with errors'
      notify("Task #{opts['taskName']} completed #{status}")
    end

    def onError(opts)
      notify("Task #{opts['taskName']} failed: #{opts['error']}", :error)
    end

    private

    def notify(message, level = :info)
      prefix = case level
               when :error then '❌'
               else '✅'
               end
      puts "[Ruby Notifier] #{prefix} #{message}"
    end
  end
end 