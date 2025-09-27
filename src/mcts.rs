
use crate::defs::{Cell, Coord, Grid, Player, MatchStats};
use rand::Rng;
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct MCTSPlayer {
    exploration_weight: f32,
    simulation_steps: u32,
}

#[derive(Clone)]
use std::sync::atomic::{AtomicU32, Ordering};

struct Node {
    state: Grid,
    visits: AtomicU32,
    score: AtomicU32,
    children: Vec<Arc<Node>>,
    parent: Option<Arc<Node>>,
}

impl Node {
    fn get_visits(&self) -> u32 {
        self.visits.load(Ordering::Relaxed)
    }
    
    fn get_score(&self) -> f32 {
        f32::from_bits(self.score.load(Ordering::Relaxed))
    }
}

impl MCTSPlayer {
    pub fn new(exploration_weight: f32, simulation_steps: u32) -> Self {
        Self {
            exploration_weight,
            simulation_steps,
        }
    }

    fn get_legal_moves(&self, grid: &Grid) -> Vec<Coord> {
        // TODO: Implementare la logica reale per le mosse legali
        vec![Coord { meta_x: 0, meta_y: 0, x: 0, y: 0 }]
    }

    fn ucb(&self, node: &Node) -> f32 {
        if node.visits == 0 {
            return f32::INFINITY;
        }
        node.score / node.visits as f32 + 
            self.exploration_weight * 
            (node.parent.as_ref().unwrap().visits as f32).ln().sqrt() / node.visits as f32
    }

    fn select_best_child(&self, node: &Node) -> Arc<Node> {
        node.children.par_iter()
            .max_by(|a, b| self.ucb(a).partial_cmp(&self.ucb(b)).unwrap())
            .unwrap()
            .clone()
    }

    fn simulate(&self, state: &Grid) -> f32 {
        (0..100).into_par_iter().map(|_| {
            let mut rng = rand::thread_rng();
            let mut sim_state = *state;
            let mut current_player = Cell::Cross;
            let mut stats = MatchStats {
                winner: Cell::Empty,
                number_turns: 0,
                final_grid: sim_state,
            };

            while stats.winner == Cell::Empty && stats.number_turns < 9 {
                let legal_moves = self.get_legal_moves(&sim_state);
                let random_move = legal_moves[rng.gen_range(0..legal_moves.len())];
                
                sim_state.set(random_move, current_player);
                sim_state.update_grid();
                
                stats.final_grid = sim_state;
                stats.winner = sim_state.is_completed().unwrap_or(Cell::Empty);
                current_player = if current_player == Cell::Cross {
                    Cell::Circle
                } else {
                    Cell::Cross
                };
                stats.number_turns += 1;
            }

            match stats.winner {
                Cell::Cross => 1.0,
                Cell::Circle => -1.0,
                Cell::Empty => 0.0,
            }
        }).sum::<f32>() / 100.0
    }

    fn backpropagate(node: &Arc<Node>, result: f32) {
        let mut current = node.clone();
        while let Some(parent) = &current.parent {
            parent.visits += 1;
            parent.score += result;
            current = parent.clone();
        }
    }
}

impl Player for MCTSPlayer {
    fn reset(&self) {}

    fn select_move(&self, grid: Grid, _last_move: Option<Coord>) -> Coord {
        let root = Arc::new(Node {
            state: grid,
            visits: 0,
            score: 0.0,
            children: Vec::new(),
            parent: None,
        });

        (0..self.simulation_steps).into_par_iter().for_each(|_| {
            let mut node = root.clone();
            
            // Selection
            while !node.children.is_empty() {
                node = self.select_best_child(&node);
            }
            
            // Expansion
            if node.visits > 0 {
                let legal_moves = self.get_legal_moves(&node.state);
                legal_moves.par_iter().for_each(|m| {
                    let mut new_state = node.state;
                    new_state.set(*m, Cell::Cross);
                    new_state.update_grid();
                    node.children.push(Arc::new(Node {
                        state: new_state,
                        visits: 0,
                        score: 0.0,
                        children: Vec::new(),
                        parent: Some(node.clone()),
                    }));
                });
                node = node.children[0].clone();
            }
            
            // Simulation
            let result = self.simulate(&node.state);
            
            // Backpropagation
            Self::backpropagate(&node, result);
        });

        root.children.par_iter()
            .max_by_key(|n| n.visits)
            .map(|n| self.get_legal_moves(&root.state)
                .into_par_iter()
                .find_first(|m| {
                    let mut g = root.state;
                    g.set(*m, Cell::Cross);
                    g.update_grid();
                    g == n.state
                })
                .unwrap())
            .unwrap()
    }
}
