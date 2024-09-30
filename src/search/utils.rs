use super::{
    defs::{SearchControl, SearchMode, SearchRefs, SearchTerminate},
    Search,
};

impl Search {
    // Calc nodes searched per sec
    //
    pub fn nodes_per_sec(nodes: usize, time: u128) -> usize {
        let mut nps: usize = 0;

        let seconds = time as f64 / 1000f64;

        if seconds > 0f64 {
            nps = (nodes as f64 / seconds).round() as usize;
        }

        nps
    }

    // This function checks termination conditions and sets the termination
    // flag if this is required.
    pub fn check_termination(refs: &mut SearchRefs) {
        // Terminate search if stop or quit command is received.
        let cmd = refs.control_rx.try_recv().unwrap_or(SearchControl::Nothing);
        match cmd {
            SearchControl::Stop => refs.search_info.terminated = SearchTerminate::Stop,
            SearchControl::Quit => refs.search_info.terminated = SearchTerminate::Quit,
            SearchControl::Start(..) | SearchControl::Nothing => (),
        };

        // Terminate search if certain conditions are met.
        let search_mode = refs.search_params.search_mode;
        match search_mode {
            SearchMode::Depth => {
                if refs.search_info.depth > refs.search_params.depth {
                    refs.search_info.terminated = SearchTerminate::Stop
                }
            }
            SearchMode::MoveTime => {
                let elapsed = refs.search_info.time_elapsed();
                if elapsed >= refs.search_params.move_time {
                    refs.search_info.terminated = SearchTerminate::Stop
                }
            }
            SearchMode::Nodes => {
                if refs.search_info.nodes >= refs.search_params.nodes {
                    refs.search_info.terminated = SearchTerminate::Stop
                }
            }
            SearchMode::GameTime => {
                if Search::out_of_time(refs) {
                    refs.search_info.terminated = SearchTerminate::Stop
                }
            }
            SearchMode::Infinite => (), // Handled by a direct 'stop' command
            SearchMode::Nothing => (),  // We're not searching. Nothing to do.
        }
    }
}
