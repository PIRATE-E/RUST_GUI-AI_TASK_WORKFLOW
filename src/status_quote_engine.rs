pub struct StatusQuotesEngine;

pub enum AgentPhase {
    InitialPlanning,
    ComplexityAnalysis,
    TaskExecution,
    ToolExecution,
    ParameterGeneration,
    Finalizing,
    ErrorRecovery,
    Idle,
}

const PLANNING_QUOTES: &[&str] = &[
    "Putting on my thinking cap... Breaking down your request into bite-sized tasks!",
    "🎯 Analyzing your request like a detective with a magnifying glass...",
    "📋 Creating a master plan... Even Napoleon would be impressed!",
    "🎨 Crafting an execution strategy so elegant, it belongs in a museum!",
];

const EXECUTION_QUOTES: &[&str] = &[
    "⚡ Rolling up my sleeves and getting to work... This is where the magic happens!",
    "🛠️ Executing tasks with the precision of a Swiss watch... Tick tock!",
    "🚀 Launching into action... Houston, we have liftoff!",
    "⚙️ Turning gears and making things happen... Like a well-oiled machine!",
];

const FINALIZING_QUOTES: &[&str] = &[
    "🎉 Grand finale time! Putting together the masterpiece...",
    "📝 Weaving together all the threads into a beautiful tapestry...",
    "✨ Ta-da! Synthesizing final response...",
];

impl StatusQuotesEngine {
    #[inline]
    pub fn generate(
        agent_phase: AgentPhase,
        task_id: Option<i32>,
        tool_name: Option<&str>,
        turn_count: u32,
    ) -> String {
        match agent_phase {
            AgentPhase::InitialPlanning => {
                PLANNING_QUOTES[(turn_count as usize) % PLANNING_QUOTES.len()]
            }
            AgentPhase::TaskExecution => {
                let base = EXECUTION_QUOTES[(turn_count as usize) % EXECUTION_QUOTES.len()];
                if let Some(id) = task_id {
                    return format!("{base} [Task {id}]");
                }
                base
            }
            AgentPhase::ToolExecution => {
                let tool = tool_name.unwrap_or("tool");
                return format!("🎪 Unleashing the {tool} tool... Watching the sparks fly!");
            }
            AgentPhase::Finalizing => {
                FINALIZING_QUOTES[(turn_count as usize) % FINALIZING_QUOTES.len()]
            }
            _ => "Thinking...",
        }
        .to_string()
    }
}
