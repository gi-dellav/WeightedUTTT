
use crate::defs::{Cell, Coord, Grid, Player, MatchStats};
use rand::Rng;
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub struct MCTSPlayer {
    exploration_weight: f32,
    simulation_steps: u32,
}

use std::sync::atomic::{AtomicU32, Ordering};

struct Node {
    state: Grid,
    visits: AtomicU32,
    score: AtomicU32,
    children: std::sync::Mutex<Vec<Arc<Node>>>,
    parent: Option<Arc<Node>>,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            state: self.state,
            visits: AtomicU32::new(self.visits.load(Ordering::Relaxed)),
            score: AtomicU32::new(self.score.load(Ordering::Relaxed)),
            children: std::sync::Mutex::new(self.children.lock().unwrap().clone()),
            parent: self.parent.clone(),
        }
    }
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
        let mut moves = Vec::new();
        // Temporary implementation to get compilation working
        for meta_x in 0..3 {
            for meta_y in 0..3 {
                for x in 0..3 {
                    for y in 0..3 {
                        if grid.matrix[(meta_x + meta_y * 3) as usize].matrix[(x + y * 3) as usize] == Cell::Empty {
                            moves.push(Coord { meta_x, meta_y, x, y });
                        }
                    }
                }
            }
        }
        moves
    }

    fn ucb(&self, node: &Node) -> f32 {
        let visits = node.visits.load(Ordering::Relaxed);
        if visits == 0 {
            return f32::INFINITY;
        }
        let score = f32::from_bits(node.score.load(Ordering::Relaxed));
        let parent_visits = node.parent.as_ref().unwrap().visits.load(Ordering::Relaxed) as f32;
        
        score / visits as f32 + 
            self.exploration_weight * 
            (parent_visits.ln().sqrt()) / visits as f32
    }

    fn select_best_child(&self, node: &Node) -> Arc<Node> {
        node.children.lock().unwrap().par_iter()
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
            parent.visits.fetch_add(1, Ordering::Relaxed);
            parent.score.fetch_add(result.to_bits(), Ordering::Relaxed);
            current = parent.clone();
        }
    }
}

impl Player for MCTSPlayer {
    fn reset(&self) {}

    fn select_move(&self, grid: Grid, _last_move: Option<Coord>) -> Coord {
        let root = Arc::new(Node {
            state: grid,
            visits: AtomicU32::new(0),
            score: AtomicU32::new(0.0f32.to_bits()),
            children: std::sync::Mutex::new(Vec::new()),
            parent: None,
        });

        (0..self.simulation_steps).into_par_iter().for_each(|_| {
            let mut node = root.clone();
            
            // Selection
            while !node.children.lock().unwrap().is_empty() {
                node = self.select_best_child(&node);
            }
            
            // Expansion
            if node.visits.load(Ordering::Relaxed) > 0 {
                let legal_moves = self.get_legal_moves(&node.state);
                legal_moves.par_iter().for_each(|m| {
                    let mut new_state = node.state;
                    new_state.set(*m, Cell::Cross);
                    new_state.update_grid();
                    node.children.lock().unwrap().push(Arc::new(Node {
                        state: new_state,
                        visits: AtomicU32::new(0),
                        score: AtomicU32::new(0.0f32.to_bits()),
                        children: std::sync::Mutex::new(Vec::new()),
                        parent: Some(node.clone()),
                    }));
                });
                node = node.children.lock().unwrap()[0].clone();
            }
            
            // Simulation
            let result = self.simulate(&node.state);
            
            // Backpropagation
            Self::backpropagate(&node, result);
        });

        root.children.lock().unwrap().par_iter()
            .max_by(|a, b| a.visits.load(Ordering::Relaxed).cmp(&b.visits.load(Ordering::Relaxed)))
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
