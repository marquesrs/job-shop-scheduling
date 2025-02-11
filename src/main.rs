use rand::distr::Distribution;
use rand::{random_range, Rng};
use std::fs::{self, OpenOptions};
use std::path::Path;
use std::io::prelude::*;

type Task = i32;

#[derive(Clone)]
pub struct Machine {
    tasks: Vec<Task>,
}

impl Machine {
    pub fn makespan(&self) -> i32 {
        let mut acc = 0i32;
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

type MachineId = usize;

pub struct MachineGroup {
    machines: Vec<Machine>,
    most_loaded: MachineId,
}

impl MachineGroup {
    pub fn new(mach_count: usize) -> Self {
        let mut group = MachineGroup {
            machines: Vec::new(),
            most_loaded: 0,
        };
        for _ in 0..mach_count {
            group.machines.push(Machine::new());
        }
        return group;
    }

    pub fn tasks_count(&self) -> usize {
        let mut count : usize = 0;
        for mach in &self.machines {
            count += mach.tasks.len();
        }
        return count;
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

    pub fn min_makespan_machine(&self) -> usize {
        let mut min = i32::MAX;
        let mut min_idx = 0;
        for i in 0..self.machines.len() {
            if self.machines[i].makespan() < min {
                min = self.machines[i].makespan();
                min_idx = i;
            }
        }
        return min_idx;
    }

    pub fn select_neighbor_rng(&self, current: MachineId) -> usize {
        assert!(self.machines.len() > 1, "Cannot pick random neighbor without at least 2 machines");
        loop {
            let neighbor = random_range(0..self.machines.len());
            if neighbor != current {
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

    fn pop_task(&mut self, mach_id: MachineId, task_id: usize) -> Option<Task> {
        if task_id >= self.machines[mach_id].tasks.len() {
            return None;
        }
        return Some(self.machines[mach_id].tasks.swap_remove(task_id));
    }

    fn push_task(&mut self, mach_id: MachineId, task: Task) {
        self.machines[mach_id].tasks.push(task);
        if self.machines[mach_id].makespan() > self.machines[self.most_loaded].makespan() {
            self.most_loaded = mach_id;
        }
    }


    fn update_most_loaded(&mut self){
        let mut max_id = 0;
        let mut max_span = 0;
        for mach_id in 0..self.machines.len() {
            let mach_span =self.machines[mach_id].makespan();
            if mach_span > max_span {
                max_span = mach_span;
                max_id = mach_id;
            }
        }
        self.most_loaded = max_id;
    }

    pub fn transfer_task(
        &mut self,
        source_id: MachineId,
        dest_id: MachineId,
        task_id: usize,
    ) -> bool {
        let res = match self.pop_task(source_id, task_id) {
            Some(e) => {
                self.push_task(dest_id, e);
                true
            }
            None => false,
        };
        self.update_most_loaded();
        return res;
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
    makespan: i32,
    task: i32,
    neighbor_id: usize,
    neighbor_makespan: i32,
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

        let source_id = mg.most_loaded;

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

const EULER : f64 = 2.718281828459045;

fn simulated_annealing(
    mg: &mut MachineGroup,
    alpha: f64,
    makespan_threshold: usize,
) -> usize {
    if mg.machines.len() < 2 {
        return 0;
    }

    let mut iter = 0;

    let mut current_temp = 1000.0;

    loop {
        let source_id = mg.most_loaded;

        let dest_id = mg.select_neighbor_rng(source_id); // TODO: Not 100% random, proportional to temp

        let (_, task_id) = match mg.peek_highest_task(source_id) {
            Some(e) => e,
            None => break,
        };

        let accept_swap = annealing_swap(mg, source_id, dest_id, task_id, current_temp);

        if accept_swap {
            mg.transfer_task(source_id, dest_id, task_id);
        }
        
        current_temp = current_temp * alpha;
        iter += 1;
        let current_makespan = mg.group_max_makespan();

        if current_makespan <= makespan_threshold { println!("Reached threshold after {}", iter); break; }
        if current_temp < 1e-18  { println!("Temp too small after {}", iter); break }
    }
    return iter;
}

fn annealing_swap(
    mg: &mut MachineGroup,
    source_id: MachineId,
    neighbor_id: MachineId,
    task_id: usize,
    temp: f64,
) -> bool {
    // let delta = (current_makespan - neighbor_makespan) as f64;
    let dist = rand::distr::Uniform::new(0.0, 1.0).unwrap();

    let old_max = mg.machines[mg.most_loaded].makespan() as f64;
    mg.transfer_task(source_id, neighbor_id, task_id);
    let new_max = mg.machines[mg.most_loaded].makespan() as f64;

    let delta = new_max - old_max;

    // Energy has been reduced, nice
    if delta < 0.0 {
        return true;
    }
    else {
        let r = dist.sample(&mut rand::rng());
        let p = EULER.powf(-delta / (temp));    
        if r < p {
            // Accept swap regardless of delta energy
            return true;
        }
        
        // Revert change
        let last_id = mg.machines[neighbor_id].tasks.len() - 1;
        mg.transfer_task(neighbor_id, source_id, last_id);
        return false;
    }
}

fn log_string(
    heuristic: Heuristic,
    n: usize,
    m: usize,
    rep: usize,
    time: usize,
    iter: usize,
    makespan: usize,
    parameter: usize,
) -> String {
    let heuristic_name = match heuristic {
        Heuristic::SimulatedAnnealing => "SimulatedAnnealing",
        Heuristic::LocalSearchBest => "LocalSearchBest",

    };
    let parameter = if heuristic == Heuristic::LocalSearchBest {String::from("N/A")} else {parameter.to_string()};

    return format!("{heuristic_name}, {n}, {m}, {rep}, {time}, {iter}, {makespan}, {parameter}\n");
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
        let header = "heuristica,n,m,replicacao,tempo_ns,iteracoes,makespan,parametro\n";
        file.write_all(header.as_bytes()).expect("BURRO");
    }

    for record in log {
        file.write_all(record.as_bytes()).expect("BURRO");
    }

}

#[derive(PartialEq, Copy, Clone)]
enum Heuristic {
    SimulatedAnnealing,
    LocalSearchBest,
}

pub fn main() {
    const M: [usize; 1] = [20];
    const R: [f32; 1] = [2.0];
    let heuristic = Heuristic::SimulatedAnnealing;
    const PRINT_INFO: bool = false;

    let mut log: Vec<String> = Vec::new();
    let mut rng = rand::rng();

    const MACH : i32 = 20;
    const TASKS : i32 = 400;

    let mut group = MachineGroup::new(20);

    for _ in 0..TASKS {
        group.push_task(0, rng.random_range(1..=100) as Task);
    }
    
    simulated_annealing(&mut group, 0.998, 1000);
    display_group(&group);

    write_log(log);
}


    // 'outer: for m in M {
    //     let mut group = MachineGroup::new(m);
    //     for r in R {
    //         let mut rep = 1;
    //         let rep_max = 1000;
    //         while rep <= rep_max {
    //             println!("{}/{}", rep, rep_max);
    //             let m = m as f32;
    //             let n = rep;
    //             // let n = m.powf(r).ceil() as usize;
    //             for _ in 0..n {
    //                 group.push_task(0, rng.random_range(1..=100) as Task);
    //             }
    //             let mut iter: usize = 0;
    //             let time_now = time::Instant::now();
       
    //             match heuristic {
    //                 Heuristic::SimulatedAnnealing => {
    //                     let alpha = 0.8;
    //                     iter = simulated_annealing(&mut group, alpha, PRINT_INFO);    
    //                 },
    //                 Heuristic::LocalSearchBest => {
    //                     iter = local_search_best(&mut group, PRINT_INFO);
    //                 }
    //             };

    //             let elapsed = time_now.elapsed().as_nanos();
    //             if PRINT_INFO { display_group(&group); }

    //             log.push(log_string(
    //                 heuristic,
    //                 n,
    //                 m as usize,
    //                 rep,
    //                 elapsed as usize,
    //                 iter,
    //                 group.group_max_makespan(),
    //                 0,
    //             ));

    //             rep += 10;
    //         }
    //     }
    // }
