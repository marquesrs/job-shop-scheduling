use rand::Rng;

type Task = u32;

type MachineId = usize;

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
		Self {
			tasks: Vec::new(),
		}
	}
	
	pub fn from_tasks(tasks: Vec<Task>) -> Self {
		Self {
			tasks: tasks,
		}
	}
}

pub struct MachineGroup {
    machines: Vec<Machine>,    
}

impl MachineGroup {
    pub fn new(mach_count: usize) -> Self {
        let mut group= MachineGroup {
            machines: Vec::new(),
        };
        for _ in 0..mach_count {
            group.machines.push(Machine::new());
        }
        return group;
    }

    pub fn group_makespan(&self) -> u32 {
        let mut max_span = 0;
        for mach in &self.machines {
            max_span = std::cmp::max(max_span, mach.makespan());
        }
        return max_span;
    }
    
    pub fn max_makespan_machine(&self) -> MachineId {
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
    
    pub fn min_makespan_machine(&self) -> MachineId {
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
    
    /*
        TODO: 
        Sempre retorna o vizinho à direita.
        Considerar retornar o vizinho mais livre ou mesmo ambos os vizinhos.
    */
    pub fn neighbor_id(&self, target: MachineId) -> Option<MachineId> {
        let neighbor = target + 1;
        if neighbor > self.machines.len() {
            return None;
        }
        else {
            return Some(neighbor);
        }
    }

    pub fn peek_last_task(&self, mach_id: MachineId) -> Option<Task> {
        let n = self.machines[mach_id].tasks.len();
        if n == 0 {
            return None;
        }
        return Some(self.machines[mach_id].tasks[n - 1]);
    }
    
    pub fn peek_highest_task(&self, mach_id: MachineId) -> Option<(Task, usize)> {
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
        
        return Some(
            (self.machines[mach_id].tasks[max_idx], max_idx)
        );
    }
    
    fn pop_last_task(&mut self, mach_id: MachineId) -> Option<Task> {
        return self.machines[mach_id].tasks.pop();
    }
    
    fn pop_task(&mut self, mach_id: MachineId, task_id: usize) -> Option<Task> {
        if task_id >= self.machines[mach_id].tasks.len() {
            return None;
        }
        return Some(self.machines[mach_id].tasks.swap_remove(task_id));
    }
    
    fn push_task(&mut self, mach_id: MachineId, task: Task){
        self.machines[mach_id].tasks.push(task);
    }
    
    pub fn transfer_last_task(
        &mut self, 
        source_id: MachineId, 
        dest_id: MachineId
    ) -> bool {
        match self.pop_last_task(source_id) {
            Some(e) => {self.push_task(dest_id, e); true}
            None => false
        }
    }
    
    pub fn transfer_task(
        &mut self, 
        source_id: MachineId, 
        dest_id: MachineId, 
        task_id: usize
    ) -> bool {
        match self.pop_task(source_id, task_id) {
            Some(e) => {self.push_task(dest_id, e); true}
            None => false
        }
    }
}

pub fn display_group(group: &MachineGroup){
    println!("group makespan: {}", group.group_makespan());
    for mach_id in 0..group.machines.len() {
        print!(
            "Makespan: {} M{}: ", 
            group.machines[mach_id].makespan(), 
            mach_id
        );
        for task in &group.machines[mach_id].tasks {
            print!("{} ", task);   
        }
        println!("\n");
    }
}

pub fn display_info(
    machine: MachineId,
    makespan: u32,
    task: u32,
    neighbor_id: usize,
    neighbor_makespan: u32,
    
) {
	println!(
	    "Machine M{} \nMakespan: {}\nTask Time: {}\nNeighbor M{}\nNeighbor makespan: {}\n", 
	    machine+1, 
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
fn local_search_best(mg: &mut MachineGroup) {
    if mg.machines.len() < 2 {
        return;
    }
    
    loop {
        let source_id = mg.max_makespan_machine();
  
        let (task, task_id) = match mg.peek_highest_task(source_id) {
            Some(e) => e,
            None => break,
        };
        
        let dest_id = mg.min_makespan_machine();
        
        let source_makespan = mg.machines[source_id].makespan(); 
        
        let dest_makespan = mg.machines[dest_id].makespan();
        
        if task + dest_makespan <  source_makespan {
            display_info(source_id, source_makespan, task, dest_id, dest_makespan);
            mg.transfer_task(source_id, dest_id, task_id);
        }
        else {
            break;
        }   
    }
}

// PEGO A MÁQUINA COM MAIOR MAKESPAN (1)
// GUARDO O MAKESPAN (16)
// PEGO UMA TASK DO TOPO (5)
// TASK.TIME(5) + MAC_VIZINHA.MAKESPAN(0) < MAKESPAN(16) 
// TRUE: TRANSFIRO A TASK PARA A MÁQUINA VIZINHA
// FALSE: ENCERRA O LOOP
fn local_search_first(mg: &mut MachineGroup) {
    if mg.machines.len() < 2 {
        return;
    }
    
    loop {
        let source_id = mg.max_makespan_machine();
    
        let dest_id = match mg.neighbor_id(source_id){
            Some(id) => id,
            None => break,
        };
        
        let task = match mg.peek_last_task(source_id) {
            Some(e) => e,
            None => break,
        };
        
        let source_makespan = mg.machines[source_id].makespan(); 
        
        let dest_makespan = mg.machines[dest_id].makespan();
        
        if task + dest_makespan <  source_makespan {
            display_info(source_id, source_makespan, task, dest_id, dest_makespan);
            mg.transfer_last_task(source_id, dest_id);
        }
        else {
            break;
        }   
    }
}

pub fn main(){
    const M: [usize; 3] = [10, 20, 50];
    const R: [f32; 2] = [1.5, 2.0];

    let mut rng = rand::rng();

    let mut group;
    let mut tasks: Vec<Task> = Vec::new();
    
    for m in M {
        group = MachineGroup::new(m);
        for r in R {
            let m = m as f32;
            let n = m.powf(r).ceil() as usize;
            for i in 0..n {
                tasks.push(rng.random_range(1..=100));
            }
        }
    }
    
    group = MachineGroup::new(10);
    for i in 0..32 {
        tasks.push(rng.random_range(1..=10))
    }
    
    
    for task in tasks {
        group.push_task(0, task);
    }
    display_group(&group);
    
    //local_search_first(&mut group);
    local_search_best(&mut group);
    
    display_group(&group);
}
