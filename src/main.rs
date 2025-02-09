use rand::{random_range, Rng};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::{io, time};
use std::io::prelude::*;
use std::time::Instant;
use std::fs::metadata;

type Task = u32;

#[derive(Clone)]
pub struct Machine {
    tasks: Vec<Task>,
}

impl Machine {
    pub fn makespan(&self) -> u32 {
        let mut acc = 0u32;
        for task in &self.tasks {
            acc += *task;
        }
        return acc;
    }

    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn from_tasks(tasks: Vec<Task>) -> Self {
        Self { tasks: tasks }
    }
}

pub struct MachineGroup {
    machines: Vec<Machine>,
}

impl MachineGroup {
    pub fn new(mach_count: usize) -> Self {
        let mut group = MachineGroup {
            machines: Vec::new(),
        };
        for _ in 0..mach_count {
            group.machines.push(Machine::new());
        }
        return group;
    }

    pub fn replace_machine_list(&mut self, machines: Vec<Machine>) {
        self.machines = machines;
    }

    pub fn machines_clone(&self) -> Vec<Machine> {
        return self.machines.clone();
    }

    pub fn group_max_makespan(&self) -> usize {
        let mut max_span = 0;
        for mach in &self.machines {
            max_span = std::cmp::max(max_span, mach.makespan());
        }
        return max_span as usize;
    }

    pub fn max_makespan_machine(&self) -> usize {
        let mut max = 0;
        let mut max_idx = 0;
        for i in 0..self.machines.len() {
            if self.machines[i].makespan() > max {
                max = self.machines[i].makespan();
                max_idx = i;
            }
        }
        return max_idx;
    }

    pub fn min_makespan_machine(&self) -> usize {
        let mut min = u32::MAX;
        let mut min_idx = 0;
        for i in 0..self.machines.len() {
            if self.machines[i].makespan() < min {
                min = self.machines[i].makespan();
                min_idx = i;
            }
        }
        return min_idx;
    }

    pub fn select_neighbor_rng(&self, mach_id: usize) -> usize {
        assert!(self.machines.len() > 1, "Cannot pick random neighbor without at least 2 machines");
        loop {
            let neighbor = random_range(0..self.machines.len());
            if neighbor != mach_id {
                return neighbor;
            }
        }
    }

    pub fn peek_highest_task(&self, mach_id: usize) -> Option<(Task, usize)> {
        let n = self.machines[mach_id].tasks.len();
        if n == 0 {
            return None;
        }
        let mut max = 0;
        let mut max_idx = 0;
        for i in 0..self.machines[mach_id].tasks.len() {
            if self.machines[mach_id].tasks[i] > max {
                max = self.machines[mach_id].tasks[i];
                max_idx = i;
            }
        }

        return Some((self.machines[mach_id].tasks[max_idx], max_idx));
    }

    fn pop_task(&mut self, mach_id: usize, task_id: usize) -> Option<Task> {
        if task_id >= self.machines[mach_id].tasks.len() {
            return None;
        }
        return Some(self.machines[mach_id].tasks.swap_remove(task_id));
    }

    fn push_task(&mut self, mach_id: usize, task: Task) {
        self.machines[mach_id].tasks.push(task);
    }

    pub fn transfer_task(
        &mut self,
        source_id: usize,
        dest_id: usize,
        task_id: usize,
    ) -> bool {
        match self.pop_task(source_id, task_id) {
            Some(e) => {
                self.push_task(dest_id, e);
                true
            }
            None => false,
        }
    }
}

pub fn display_group(group: &MachineGroup) {
    println!("Group Max Makespan: {}", group.group_max_makespan());
    for mach_id in 0..group.machines.len() {
        print!(
            "Makespan: {} M{}: ",
            group.machines[mach_id].makespan(),
            mach_id
        );
        for task in &group.machines[mach_id].tasks {
            print!("{} ", task);
        }
        println!("");
    }
}

pub fn display_info(
    machine: usize,
    makespan: u32,
    task: u32,
    neighbor_id: usize,
    neighbor_makespan: u32,
) {
    println!(
        "Machine M{} \nMakespan: {}\nTask Time: {}\nNeighbor M{}\nNeighbor makespan: {}\n",
        machine + 1,
        makespan,
        task,
        neighbor_id,
        neighbor_makespan
    );
}

// PEGO A MÁQUINA COM MAIOR MAKESPAN
// PEGO DELA A TAREFA COM MAIOR TIME
// PEGO A MÁQUINA COM MENOR MAKESPAN
// TASK.TIME + MAC_VIZINHA.MAKESPAN < MAKESPAN
// TRUE: TRANSFIRO A TASK PARA A MÁQUINA VIZINHA
// FALSE: ENCERRA O LOOP
fn local_search_best(mg: &mut MachineGroup, print_info: bool) -> usize {
    if mg.machines.len() < 2 {
        return 0;
    }
    let mut iter = 0;
    loop {
        iter = iter + 1;

        let source_id = mg.max_makespan_machine();

        let (task, task_id) = match mg.peek_highest_task(source_id) {
            Some(e) => e,
            None => break,
        };

        let dest_id = mg.min_makespan_machine();

        let source_makespan = mg.machines[source_id].makespan();

        let dest_makespan = mg.machines[dest_id].makespan();

        if task + dest_makespan < source_makespan {
            if print_info {
                display_info(
                    source_id, 
                    source_makespan, 
                    task, 
                    dest_id, 
                    dest_makespan
                );
            }
            mg.transfer_task(source_id, dest_id, task_id);
        } else {
            break;
        }
    }
    return iter;
}

fn simulated_annealing(
    mg: &mut MachineGroup, 
    alpha: f64, 
    print_info: bool
) -> usize {
    if mg.machines.len() < 2 {
        return 0;
    }

    let mut current_makespan = mg.group_max_makespan();
    let mut best_makespan = current_makespan;
    let mut best_group = mg.machines_clone();

    let mut iter = 0;
    let mut temperature = 1.0; 

    loop {
        iter = iter + 1;
        if iter > MAX_ITER {
            break;
        }

        let source_id = mg.max_makespan_machine();

        let dest_id = mg.select_neighbor_rng(source_id);

        let (task, task_id) = match mg.peek_highest_task(source_id) {
            Some(e) => e,
            None => break,
        };

        let source_makespan = mg.machines[source_id].makespan();

        let dest_makespan = mg.machines[dest_id].makespan();

        let prob = accept_probability(source_makespan, task + dest_makespan, iter);

        if rand::rng().random_range(0.0..=1.0) < prob {
            mg.transfer_task(source_id, dest_id, task_id);
            current_makespan = mg.group_max_makespan();
            if current_makespan < best_makespan {
                if print_info {
                    display_info(
                        source_id, 
                        source_makespan, 
                        task, 
                        dest_id, 
                        dest_makespan
                    );
                }                
                best_makespan = current_makespan;
                best_group = mg.machines_clone();
            }
        }

        temperature = temperature * alpha;
        if temperature < 0.01 {
            break;
        }
    }
    mg.replace_machine_list(best_group);
    return iter;
}

fn temperature(exec_progress: f64) -> f64 {
    const FALLOFF : f64 = 0.8;
    return -exec_progress + 1.0;
}

fn accept_probability(
    current_makespan: u32, 
    neighbor_makespan: u32, 
    k: usize, 
) -> f64 {
    let exec_progress = (k as f64) / (MAX_ITER as f64);
        
    if current_makespan > neighbor_makespan {
        return 1.0 - temperature(exec_progress);
    }
    else {
        return temperature(exec_progress);
    }
}

fn log_string(
    is_blm: bool,
    n: usize,
    m: usize,
    rep: usize,
    time: usize,
    iter: usize,
    makespan: usize,
    parameter: usize,
) -> String {
    let heuristic = if is_blm {
        "Local Search Best Improvement"
    } else { "Simulated Annealing" };
    let parameter = if is_blm {String::from("0")} else {parameter.to_string()};

    return format!("{heuristic}, {n}, {m}, {rep}, {time}, {iter}, {makespan}, {parameter}\n");
}

fn write_log(log: Vec<String>) {
    let file_path = "log.txt";
    let file_exists = Path::new(file_path).exists();

    let mut file = OpenOptions::new()
        .append(true)    
        .create(true)    
        .open(file_path)
        .expect("BURRO");

    let file_metadata = fs::metadata(file_path).expect("BURRO");

    if !file_exists || file_metadata.len() == 0 {
        let header = "heuristica, n, m, replicacao, tempo, iteracoes, valor, parametro\n";
        file.write_all(header.as_bytes()).expect("BURRO");
    }

    for record in log {
        file.write_all(record.as_bytes()).expect("BURRO");
    }
    
}

const MAX_ITER: usize = 1000;

pub fn main() {
    const M: [usize; 3] = [10, 20, 50];
    const R: [f32; 2] = [1.5, 2.0];
    const IS_BLM: bool = false;
    const PRINT_INFO: bool = true;
    
    let mut log: Vec<String> = Vec::new();

    let mut rng = rand::rng();

    let mut group = MachineGroup::new(1);
    
    'outer: for m in M {
        group = MachineGroup::new(m);
        for r in R {
            for rep in 0..10 {
                let m = m as f32;
                let n = m.powf(r).ceil() as usize;
                for _ in 0..n {
                    group.push_task(0, rng.random_range(1..=100) as Task);
                }
                let mut iter: usize = 0;
                let time_now = time::Instant::now();
                if IS_BLM { 
                    iter = local_search_best(&mut group, PRINT_INFO); 
                }
                else {
                    let alpha = 0.8; 
                    iter = simulated_annealing(&mut group, alpha, PRINT_INFO);
                }
                let elapsed = time_now.elapsed().as_millis();
                if PRINT_INFO { display_group(&group); }
                log.push(log_string(
                    IS_BLM, 
                    n, 
                    m as usize, 
                    rep, 
                    elapsed as usize, 
                    iter, 
                    group.group_max_makespan(),
                    0,
                ));
                break 'outer; // TODO: REMOVER
            }
        }
    }
    
    write_log(log);
}
